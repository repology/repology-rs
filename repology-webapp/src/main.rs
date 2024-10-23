// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod config;

use anyhow::{Context, Error};
use clap::Parser;
use sqlx::PgPool;
use tracing::info;

use crate::config::Config;
use repology_webapp::create_app;

async fn async_main() -> Result<(), Error> {
    let config = Config::parse();

    if let Some(log_directory) = &config.log_directory {
        use tracing_appender::rolling::{RollingFileAppender, Rotation};
        let logfile = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix("repology-webapp.log")
            .max_log_files(14)
            .build(log_directory)
            .context("logging initialization failed")?;
        tracing_subscriber::fmt().with_writer(logfile).init();
    } else {
        tracing_subscriber::fmt::init();
    }

    if let Some(socket_addr) = &config.prometheus_export {
        info!("initializing prometheus exporter");
        use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};

        const DURATION_SECONDS_BUCKETS: &[f64] = &[
            0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        const SIZE_BYTES_BUCKETS: &[f64] = &[
            64., 128., 256., 512., 1024., 2048., 4096., 8192., 16384., 32768., 65536., 131072.,
            262144., 524288., 1048576., 2097152.,
        ];

        PrometheusBuilder::new()
            .set_buckets_for_metric(
                Matcher::Suffix("_duration_seconds".to_string()),
                DURATION_SECONDS_BUCKETS,
            )
            .unwrap()
            .set_buckets_for_metric(
                Matcher::Suffix("_size_bytes".to_string()),
                SIZE_BYTES_BUCKETS,
            )
            .unwrap()
            .with_http_listener(*socket_addr)
            .install()
            .context("prometheus exporter initialization failed")?;

        let collector = metrics_process::Collector::default();
        collector.describe();

        tokio::spawn(async move {
            loop {
                collector.collect();
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }

    info!("initializing database pool");
    let pool = PgPool::connect(&config.dsn)
        .await
        .context("error creating PostgreSQL connection pool")?;

    info!("initializing application");
    let app = create_app(pool).await?;

    info!("listening");
    let listener = tokio::net::TcpListener::bind(&config.listen).await.unwrap();
    axum::serve(listener, app)
        .await
        .context("error starting HTTP server")
}

fn main() -> Result<(), Error> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}
