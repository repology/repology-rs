// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(duration_constructors)]
#![feature(duration_constructors_lite)]
#![feature(map_try_insert)]
#![feature(try_blocks)]
#![feature(lock_value_accessors)]
#![feature(coverage_attribute)]
#![feature(ip)]

mod checker;
mod config;
mod delayer;
mod errors;
mod feeder;
mod hosts;
mod http_client;
mod mainloop;
mod optional_semaphore;
mod queuer;
mod resolver;
mod updater;

use anyhow::{Context, Result};
use metrics::{counter, gauge};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};
use tracing::info;

use crate::config::Config;
use crate::mainloop::link_check_loop;

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
    use tracing_subscriber::Layer;
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    info!("initializing logging");

    let mut layers = vec![];

    if let Some(loki_url) = &config.loki_url {
        let (layer, task) = tracing_loki::builder()
            .label("service", "repology-linkchecker")?
            .build_url(loki_url.clone())
            .context("loki logging initialization failed")?;
        tokio::spawn(task);
        layers.push(layer.boxed());
    }

    let layer = tracing_subscriber::fmt::Layer::new().with_timer(
        tracing_subscriber::fmt::time::ChronoLocal::new(String::from("%F %T%.6f")),
    );

    if let Some(log_directory) = &config.log_directory {
        use tracing_appender::rolling::{RollingFileAppender, Rotation};
        let logfile = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix("repology-linkchecker.log")
            .max_log_files(14)
            .build(log_directory)
            .context("logging initialization failed")?;
        layers.push(layer.with_writer(logfile).boxed());
    } else {
        layers.push(layer.boxed());
    }

    tracing_subscriber::registry()
        .with(EnvFilter::new("info"))
        .with(layers)
        .init();

    Ok(())
}

fn init_metrics(config: &Config) -> Result<()> {
    if let Some(socket_addr) = &config.prometheus_export {
        info!("initializing prometheus exporter");
        use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};

        const CHECK_DURATION_SECONDS_BUCKETS: &[f64] = &[
            0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 20.0,
            40.0, 60.0, 90.0, 120.0, 180.0, 300.0, 600.0,
        ];
        const CHECK_PERIOD_SECONDS_BUCKETS: &[f64] = &[
            60.0,
            120.0,
            300.0,
            600.0,
            1200.0,
            1.0 * 3600.0,
            2.0 * 3600.0,
            4.0 * 3600.0,
            8.0 * 3600.0,
            1.0 * 86400.0,
            2.0 * 86400.0,
            4.0 * 86400.0,
            7.0 * 86400.0,
            14.0 * 86400.0,
            30.0 * 86400.0,
            61.0 * 86400.0,
            120.0 * 86400.0,
            183.0 * 86400.0,
            366.0 * 86400.0,
        ];

        PrometheusBuilder::new()
            .set_buckets_for_metric(
                Matcher::Suffix("_check_duration_seconds".to_string()),
                CHECK_DURATION_SECONDS_BUCKETS,
            )
            .unwrap()
            .set_buckets_for_metric(
                Matcher::Suffix("_overdue_age_seconds".to_string()),
                CHECK_PERIOD_SECONDS_BUCKETS,
            )
            .unwrap()
            .set_buckets_for_metric(
                Matcher::Suffix("_check_period_seconds".to_string()),
                CHECK_PERIOD_SECONDS_BUCKETS,
            )
            .unwrap()
            .set_buckets_for_metric(
                Matcher::Suffix("_recovery_duration_seconds".to_string()),
                CHECK_PERIOD_SECONDS_BUCKETS,
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
                conn.execute("SET application_name = 'repology-linkchecker'")
                    .await?;
                Ok(())
            })
        })
        .connect(&config.dsn)
        .await
        .context("error creating PostgreSQL connection pool")?;

    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse().with_context(|| "failed to process configuration")?;

    init_logging(&config).with_context(|| "failed to init logging")?;
    init_metrics(&config).with_context(|| "failed to init metrics")?;
    let pool = init_database(&config)
        .await
        .with_context(|| "failed to init database")?;

    link_check_loop(pool, config).await?;

    Ok(())
}
