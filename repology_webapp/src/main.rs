// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod badges;
mod config;
mod font;
mod package;
mod repometadata;
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
use crate::repometadata::RepositoryMetadataCache;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::parse();

    let pool = PgPool::connect(&config.dsn)
        .await
        .context("error creating PostgreSQL connection pool")?;

    let font_measurer = FontMeasurer::new();

    let repository_metadata_cache = RepositoryMetadataCache::new(pool.clone());
    repository_metadata_cache
        .update()
        .await
        .context("error getting repository metadata")?;

    let state = AppState::new(pool, font_measurer, repository_metadata_cache);

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
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.listen).await.unwrap();
    axum::serve(listener, app)
        .await
        .context("error starting HTTP server")
}
