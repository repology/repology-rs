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
mod extractors;
mod feeds;
mod font;
mod graphs;
mod handlers;
mod middleware;
mod package;
mod query;
mod repository_data;
mod result;
mod routes;
mod state;
mod static_files;
mod template_funcs;
mod xmlwriter;

use std::sync::Arc;

use anyhow::{Context, Result};
use axum::Router;
use sqlx::PgPool;
use tracing::info;

use crate::background_tasks::*;
use crate::config::AppConfig;
use crate::font::FontMeasurer;
use crate::middleware::metrics_middleware;
use crate::repository_data::RepositoriesDataCache;
use crate::state::AppState;
use crate::static_files::STATIC_FILES;

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
    let important_projects_cache = crate::handlers::get_important_projects(&pool)
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
    Ok(crate::routes::Route::to_router_with(|router| {
        router.layer(axum::middleware::from_fn(metrics_middleware))
    })
    .route_layer(tower_cookies::CookieManagerLayer::new())
    .with_state(state))
}
