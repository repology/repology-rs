// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(iterator_try_collect)]
#![feature(try_blocks)]

use anyhow::{Context, Result};
use metrics::{counter, gauge};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};
use tracing::info;

use repology_webapp::config::Config;
use repology_webapp::create_app;

#[allow(unexpected_cfgs)]
fn collect_tokio_runtime_metrics() {
    let metrics = tokio::runtime::Handle::current().metrics();

    #[cfg(tokio_unstable)]
    gauge!("tokio_blocking_queue_depth").set(metrics.blocking_queue_depth() as f64);
    #[cfg(tokio_unstable)]
    counter!("tokio_budget_forced_yield_count_total").absolute(metrics.budget_forced_yield_count());
    gauge!("tokio_global_queue_depth").set(metrics.global_queue_depth() as f64);
    gauge!("tokio_num_alive_tasks").set(metrics.num_alive_tasks() as f64);
    #[cfg(tokio_unstable)]
    gauge!("tokio_num_blocking_threads").set(metrics.num_blocking_threads() as f64);
    #[cfg(tokio_unstable)]
    gauge!("tokio_num_idle_blocking_threads").set(metrics.num_idle_blocking_threads() as f64);
    gauge!("tokio_num_workers").set(metrics.num_workers() as f64);
    #[cfg(tokio_unstable)]
    counter!("tokio_spawned_tasks_count_total").absolute(metrics.spawned_tasks_count());

    for nworker in 0..metrics.num_workers() {
        let labels = [("worker", format!("{nworker}"))];
        #[cfg(tokio_unstable)]
        gauge!("tokio_worker_local_queue_depth", &labels)
            .set(metrics.worker_local_queue_depth(nworker) as f64);
        #[cfg(tokio_unstable)]
        counter!("tokio_worker_local_schedule_count_total", &labels)
            .absolute(metrics.worker_local_schedule_count(nworker));
        #[cfg(tokio_unstable)]
        gauge!("tokio_worker_mean_poll_time", &labels)
            .set(metrics.worker_mean_poll_time(nworker).as_secs_f64());
        #[cfg(tokio_unstable)]
        counter!("tokio_worker_noop_count_total", &labels)
            .absolute(metrics.worker_noop_count(nworker));
        #[cfg(tokio_unstable)]
        counter!("tokio_worker_overflow_count_total", &labels)
            .absolute(metrics.worker_overflow_count(nworker));
        counter!("tokio_worker_park_count_total", &labels)
            .absolute(metrics.worker_park_count(nworker));
        counter!("tokio_worker_park_unpark_count_total", &labels)
            .absolute(metrics.worker_park_unpark_count(nworker));
        #[cfg(tokio_unstable)]
        counter!("tokio_worker_poll_count_total", &labels)
            .absolute(metrics.worker_poll_count(nworker));
        #[cfg(tokio_unstable)]
        counter!("tokio_worker_steal_count_total", &labels)
            .absolute(metrics.worker_steal_count(nworker));
        #[cfg(tokio_unstable)]
        counter!("tokio_worker_steal_operations_total", &labels)
            .absolute(metrics.worker_steal_operations(nworker));
        counter!("tokio_worker_total_busy_duration", &labels)
            .absolute(metrics.worker_total_busy_duration(nworker).as_secs());
    }
}

fn init_logging(config: &Config) -> Result<()> {
    info!("initializing logging");

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

    Ok(())
}

fn init_metrics(config: &Config) -> Result<()> {
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
                collect_tokio_runtime_metrics();
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }

    Ok(())
}

async fn init_database(config: &Config) -> Result<PgPool> {
    info!("initializing database pool");
    let pool = PgPoolOptions::new()
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("SET application_name = 'repology-webapp'")
                    .await?;
                conn.execute("SET search_path = repology, public").await?;
                Ok(())
            })
        })
        .connect(&config.dsn)
        .await
        .context("error creating PostgreSQL connection pool")?;

    Ok(pool)
}

async fn async_main() -> Result<()> {
    let config = Config::parse().with_context(|| "failed to process configuration")?;

    init_logging(&config).with_context(|| "failed to init logging")?;
    init_metrics(&config).with_context(|| "failed to init metrics")?;
    let pool = init_database(&config)
        .await
        .with_context(|| "failed to init database")?;

    info!("initializing application");
    let app = create_app(pool, config.app_config).await?;

    info!("listening");
    let listener = tokio::net::TcpListener::bind(&config.listen).await.unwrap();
    axum::serve(listener, app)
        .await
        .context("error starting HTTP server")
}

fn main() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}
