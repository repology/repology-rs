// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(iterator_try_collect)]

mod badges;
mod config;
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
use clap::Parser;
use sqlx::PgPool;

use crate::config::Config;
use crate::font::FontMeasurer;
use crate::repository_data::RepositoryDataCache;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::parse();

    let pool = PgPool::connect(&config.dsn)
        .await
        .context("error creating PostgreSQL connection pool")?;

    let font_measurer = FontMeasurer::new();

    let repository_data_cache = RepositoryDataCache::new(pool.clone());
    repository_data_cache
        .update()
        .await
        .context("error getting repository metadata")?;

    let state = AppState::new(pool, font_measurer, repository_data_cache);

    let app = Router::new()
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
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.listen).await.unwrap();
    axum::serve(listener, app)
        .await
        .context("error starting HTTP server")
}
