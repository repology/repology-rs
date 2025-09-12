// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(duration_constructors)]
#![feature(coverage_attribute)]

mod args;
mod cpe;
mod datetime;
mod fetcher;
mod processors;
mod status_tracker;
mod vulnupdater;

use std::cell::LazyCell;

use anyhow::{Context, Result, bail};
use clap::Parser as _;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};
use tracing::info;

use args::Args;
use fetcher::NvdFetcher;
use processors::cpe::CpeProcessor;
use processors::cve::CveProcessor;
use status_tracker::SourceUpdateStatusTracker;

use vulnupdater::{Datasource, VulnUpdater};

fn init_logging(args: &Args) -> Result<()> {
    use tracing_subscriber::Layer;
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    info!("initializing logging");

    let mut layers = vec![];

    if let Some(loki_url) = &args.loki_url {
        let (layer, task) = tracing_loki::builder()
            .label("service", "repology-vulnupdater")?
            .build_url(loki_url.clone())
            .context("loki logging initialization failed")?;
        tokio::spawn(task);
        layers.push(layer.boxed());
    }

    let layer = tracing_subscriber::fmt::Layer::new().with_timer(
        tracing_subscriber::fmt::time::ChronoLocal::new(String::from("%F %T%.6f")),
    );

    if let Some(log_directory) = &args.log_directory {
        use tracing_appender::rolling::{RollingFileAppender, Rotation};
        let logfile = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix("repology-vulnupdater.log")
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

fn init_metrics(args: &Args) -> Result<()> {
    if let Some(socket_addr) = &args.prometheus_export {
        if args.once_only {
            bail!("prometheus export is not supported in --once-only mode");
        }
        info!("initializing metrics");
        metrics_exporter_prometheus::PrometheusBuilder::new()
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

    Ok(())
}

async fn init_database(args: &Args) -> Result<PgPool> {
    info!("creating PostgreSQL pool");
    let pool = PgPoolOptions::new()
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("SET application_name = 'repology-vulnupdater'")
                    .await?;
                conn.execute("SET search_path = vulnupdater").await?;
                Ok(())
            })
        })
        .connect(&args.dsn)
        .await
        .context("postgres connection failed")?;

    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    init_logging(&args).with_context(|| "failed to init logging")?;
    init_metrics(&args).with_context(|| "failed to init metrics")?;
    let pool = init_database(&args)
        .await
        .with_context(|| "failed to init database")?;

    // make sure schema exist before migrations, so
    // _sqlx_migrations table can be placed within it
    info!("creating PostgreSQL schema");
    sqlx::query("CREATE SCHEMA IF NOT EXISTS vulnupdater")
        .execute(&pool)
        .await
        .context("schema creation failed")?;
    info!("running migrations");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("migrations failed")?;

    info!("initializing datasources");
    let cve_processor =
        LazyCell::new(|| CveProcessor::new(&pool).skip_finalization(args.no_update_repology));
    let cpe_processor =
        LazyCell::new(|| CpeProcessor::new(&pool).skip_finalization(args.no_update_repology));

    let mut datasources: Vec<Datasource> = vec![];

    if args.should_update_all() || args.should_update_cve {
        datasources.push(Datasource {
            name: "CVE",
            url: "https://services.nvd.nist.gov/rest/json/cves/2.0",
            processor: &*cve_processor,
        });
    }
    if args.should_update_all() || args.should_update_cpe {
        datasources.push(Datasource {
            name: "CPE",
            url: "https://services.nvd.nist.gov/rest/json/cpes/2.0",
            processor: &*cpe_processor,
        });
    }

    info!("initializing vulnupdater");
    let status_tracker = SourceUpdateStatusTracker::new(&pool);
    let fetcher = NvdFetcher::new()?;
    let vulnupdater = VulnUpdater::new(&status_tracker, &fetcher);

    info!("running");
    if args.once_only {
        vulnupdater.process_datasources_once(&datasources).await?;
    } else {
        vulnupdater.run_loop(&datasources, args.update_period).await;
    }

    Ok(())
}
