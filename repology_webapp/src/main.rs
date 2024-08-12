// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod badges;
mod config;
mod font;
mod result;
mod state;
mod views;
mod xmlwriter;

use axum::routing::get;
use axum::Router;
use clap::Parser;
use sqlx::PgPool;

use crate::config::Config;
use crate::font::FontMeasurer;
use crate::state::AppState;

use anyhow::{Context, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::parse();

    let pool = PgPool::connect(&config.dsn)
        .await
        .context("error creating PostgreSQL connection pool")?;

    let font_measurer = FontMeasurer::new();

    let state = AppState::new(pool, font_measurer);

    let app = Router::new()
        .route("/api/v1/project/:project_name", get(views::api_v1_project))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.listen).await.unwrap();
    axum::serve(listener, app)
        .await
        .context("error starting HTTP server")
}
