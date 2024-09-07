// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod config;

use anyhow::{Context, Error};
use clap::Parser;
use sqlx::PgPool;

use crate::config::Config;
use repology_webapp::create_app;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::parse();

    let pool = PgPool::connect(&config.dsn)
        .await
        .context("error creating PostgreSQL connection pool")?;

    let app = create_app(pool).await?;

    let listener = tokio::net::TcpListener::bind(&config.listen).await.unwrap();
    axum::serve(listener, app)
        .await
        .context("error starting HTTP server")
}
