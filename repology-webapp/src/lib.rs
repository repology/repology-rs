// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(iterator_try_collect)]

mod badges;
mod font;
mod package;
mod query;
mod repository_data;
mod result;
mod state;
mod views;
mod xmlwriter;

use anyhow::{Context, Error};
use axum::{routing::get, Router};
use sqlx::PgPool;

use crate::font::FontMeasurer;
use crate::repository_data::RepositoryDataCache;
use crate::state::AppState;

pub async fn create_app(pool: PgPool) -> Result<Router, Error> {
    let font_measurer = FontMeasurer::new();

    let repository_data_cache = RepositoryDataCache::new(pool.clone());
    repository_data_cache
        .update()
        .await
        .context("error getting repository metadata")?;

    let state = AppState::new(pool, font_measurer, repository_data_cache);

    Ok(Router::new()
        .route("/api/v1/project/:project_name", get(views::api_v1_project))
        .route(
            "/badge/tiny-repos/:project_name.svg",
            get(views::badge_tiny_repos),
        )
        .route(
            "/badge/version-for-repo/:repository_name/:project_name.svg",
            get(views::badge_version_for_repo),
        )
        .route(
            "/badge/vertical-allrepos/:project_name.svg",
            get(views::badge_vertical_allrepos),
        )
        .with_state(state))
}
