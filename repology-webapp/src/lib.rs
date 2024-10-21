// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(iterator_try_collect)]
#![feature(coverage_attribute)]
#![feature(stmt_expr_attributes)]
#![feature(assert_matches)]

mod badges;
mod constants;
mod endpoints;
mod font;
mod package;
mod query;
mod repository_data;
mod result;
mod state;
mod static_files;
mod template_context;
mod url_for;
mod views;
mod xmlwriter;

use std::time::Instant;

use anyhow::{Context, Error};
use axum::{
    extract::{MatchedPath, Request},
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
    Router,
};
use metrics::{counter, histogram};
use sqlx::PgPool;
use tracing::info;

use crate::font::FontMeasurer;
use crate::repository_data::RepositoryDataCache;
use crate::state::AppState;
use crate::static_files::STATIC_FILES;

async fn track_metrics(matched_path: MatchedPath, req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let path = matched_path.as_str().to_owned();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    counter!("repology_webapp_http_requests_total", "path" => path.clone(), "status" => status)
        .increment(1);
    histogram!("repology_webapp_http_requests_duration_seconds", "path" => path).record(latency);

    response
}

#[tracing::instrument(name = "app init", skip_all)]
pub async fn create_app(pool: PgPool) -> Result<Router, Error> {
    info!("initializing font measurer");
    let font_measurer = FontMeasurer::new();

    info!("initializing repository data cache");
    let repository_data_cache = RepositoryDataCache::new(pool.clone());
    repository_data_cache
        .update()
        .await
        .context("error getting repository metadata")?;

    let state = AppState::new(pool, font_measurer, repository_data_cache);

    info!("initializing static files");
    let _ = &*STATIC_FILES;

    info!("initializing routes");
    use crate::endpoints::Endpoint::*;
    #[rustfmt::skip]
    Ok(Router::new()
        .route(Api.path(), get(views::api_v1))
        .route(ApiV1.path(), get(views::api_v1))
        .route(ApiV1Project.path(), get(views::api_v1_project))
        .route(ApiV1Projects.path(), get(views::api_v1_projects))
        .route(ApiV1ProjectsBounded.path(), get(views::api_v1_projects_bounded))
        .route(BadgeLatestVersions.path(), get(views::badge_latest_versions))
        .route(BadgeTinyRepos.path(), get(views::badge_tiny_repos))
        .route(BadgeVersionForRepo.path(), get(views::badge_version_for_repo))
        .route(BadgeVerticalAllRepos.path(), get(views::badge_vertical_allrepos))
        .route(Docs.path(), get(views::docs))
        .route(DocsAbout.path(), get(views::docs_about))
        .route(DocsBots.path(), get(views::docs_bots))
        .route(DocsNotSupported.path(), get(views::docs_not_supported))
        .route(DocsRequirements.path(), get(views::docs_requirements))
        .route(Log.path(), get(views::log))
        .route(MaintainerProblems.path(), get(views::maintainer_problems))
        .route(News.path(), get(views::news))
        .route(ProjectVersions.path(), get(views::project_versions))
        .route(Projects.path(), get(views::projects))
        .route(ProjectsBounded.path(), get(views::projects_bounded))
        .route(RepositoryProblems.path(), get(views::repository_problems))
        .route(StaticFile.path(), get(views::static_file))
        .route(Tools.path(), get(views::tools))
        .route_layer(middleware::from_fn(track_metrics))
        .with_state(state))
}
