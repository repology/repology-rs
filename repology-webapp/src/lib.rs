// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(iterator_try_collect)]
#![feature(coverage_attribute)]
#![feature(stmt_expr_attributes)]

mod badges;
mod endpoints;
mod font;
mod package;
mod query;
mod repository_data;
mod result;
mod state;
mod static_files;
mod views;
mod xmlwriter;

use anyhow::{Context, Error};
use axum::{routing::get, Router};
use sqlx::PgPool;

use crate::font::FontMeasurer;
use crate::repository_data::RepositoryDataCache;
use crate::state::AppState;
use crate::static_files::StaticFiles;

pub async fn create_app(pool: PgPool) -> Result<Router, Error> {
    let font_measurer = FontMeasurer::new();

    let repository_data_cache = RepositoryDataCache::new(pool.clone());
    repository_data_cache
        .update()
        .await
        .context("error getting repository metadata")?;

    let static_files = StaticFiles::new();

    let state = AppState::new(pool, font_measurer, repository_data_cache, static_files);

    use crate::endpoints::Endpoint::*;
    #[rustfmt::skip]
    Ok(Router::new()
        .route(ApiV1Project.path(), get(views::api_v1_project))
        .route(BadgeTinyRepos.path(), get(views::badge_tiny_repos))
        .route(BadgeVersionForRepo.path(), get(views::badge_version_for_repo))
        .route(BadgeVerticalAllRepos.path(), get(views::badge_vertical_allrepos))
        .route(BadgeLatestVersions.path(), get(views::badge_latest_versions))
        .route(StaticFile.path(), get(views::static_file))
        .with_state(state))
}
