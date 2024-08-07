// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod state;
mod views;

use axum::routing::get;
use axum::Router;
use sqlx::PgPool;

use crate::state::AppState;
use crate::views::*;

use anyhow::{Context, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let dsn = "postgresql://repology@localhost:5433/repology";
    let pool = PgPool::connect(dsn).await?;

    let state = AppState::new(pool);

    let app = Router::new()
        .route("/api/v1/project/:name", get(api_v1_project))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .await
        .context("error starting HTTP server")
}
