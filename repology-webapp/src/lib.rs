// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(iterator_try_collect)]
#![feature(coverage_attribute)]
#![feature(stmt_expr_attributes)]
#![feature(duration_constructors)]
#![feature(lock_value_accessors)]
#![feature(iter_collect_into)]
#![feature(default_field_values)]
#![allow(clippy::module_inception)]

mod background_tasks;
mod badges;
pub mod config;
mod constants;
mod endpoints;
mod extractors;
mod feeds;
mod font;
mod graphs;
mod package;
mod query;
mod repository_data;
mod result;
mod state;
mod static_files;
mod template_context;
mod template_funcs;
mod url_for;
mod views;
mod xmlwriter;

use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use axum::{
    Router,
    body::HttpBody,
    extract::{MatchedPath, Request},
    middleware::{self, Next},
    response::IntoResponse,
};
use metrics::{counter, histogram};
use sqlx::PgPool;
use tracing::info;

use crate::background_tasks::*;
use crate::config::AppConfig;
use crate::font::FontMeasurer;
use crate::repository_data::RepositoriesDataCache;
use crate::state::AppState;
use crate::static_files::STATIC_FILES;

async fn track_metrics(matched_path: MatchedPath, req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();

    let path_for_metrics = {
        // normalize some paths which lead to the same endpoints; XXX this will hopefully be gone
        // someday when endpoints are redesigned (e.g. /projects/{bound}/ → /projects/?start=)
        let mut path = matched_path
            .as_str()
            .trim_end_matches("{bound}/")
            .trim_end_matches("/{sorting}");
        if path.starts_with("/graph/total/") {
            path = "/graph/total/..."
        }
        if path.starts_with("/graph/repo/") {
            path = "/graph/repo/..."
        }

        path.to_owned()
    };

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    counter!("repology_webapp_http_requests_total", "path" => path_for_metrics.clone(), "status" => status)
        .increment(1);
    histogram!("repology_webapp_http_requests_duration_seconds", "path" => path_for_metrics.clone())
        .record(latency);

    if let Some(body_size) = response.body().size_hint().exact() {
        histogram!("repology_webapp_http_response_size_bytes", "path" => path_for_metrics)
            .record(body_size as f64);
    }

    response
}

#[cfg_attr(
    not(feature = "coverage"),
    tracing::instrument(name = "app init", skip_all)
)]
pub async fn create_app(pool: PgPool, config: AppConfig) -> Result<Router> {
    info!("initializing font measurer");
    let font_measurer = FontMeasurer::new();

    info!("initializing repository data cache");
    let repository_data_cache = RepositoriesDataCache::new(pool.clone())
        .await
        .context("initial repository data cache fill failed")?;

    info!("initializing important projects cache");
    let important_projects_cache = crate::views::get_important_projects(&pool)
        .await
        .context("initial important projects cache fill failed")?;

    let state = Arc::new(AppState::new(
        pool.clone(),
        font_measurer,
        repository_data_cache,
        config,
        important_projects_cache,
    ));

    info!("initializing static files");
    let _ = &*STATIC_FILES;

    info!("starting background tasks");
    start_repository_data_cache_task(Arc::clone(&state));
    start_important_projects_cache_task(Arc::clone(&state), pool);

    info!("initializing routes");
    Ok(crate::endpoints::Endpoint::to_router()
        .route_layer(middleware::from_fn(track_metrics))
        .route_layer(tower_cookies::CookieManagerLayer::new())
        .with_state(state))
}
