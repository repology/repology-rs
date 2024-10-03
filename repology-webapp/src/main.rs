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

    if let Some(socket_addr) = &config.prometheus_export {
        metrics_exporter_prometheus::PrometheusBuilder::new()
            .with_http_listener(*socket_addr)
            .install()
            .context("prometheus exporter initialization failed")?;

        let collector = metrics_process::Collector::default();
        collector.describe();

        tokio::spawn(async move {
            collector.collect();
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        });
    }

    let pool = PgPool::connect(&config.dsn)
        .await
        .context("error creating PostgreSQL connection pool")?;

    let app = create_app(pool).await?;

    let listener = tokio::net::TcpListener::bind(&config.listen).await.unwrap();
    axum::serve(listener, app)
        .await
        .context("error starting HTTP server")
}
