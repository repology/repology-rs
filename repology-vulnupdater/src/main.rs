// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(duration_constructors)]

mod args;
mod cpe;
mod datetime;
mod fetcher;
mod processors;
mod status_tracker;
mod vulnupdater;

use std::cell::LazyCell;

use anyhow::{Context, Error};
use clap::Parser as _;
use sqlx::postgres::PgPoolOptions;
use sqlx::Executor;
use tracing::info;

use args::Args;
use fetcher::NvdFetcher;
use processors::cpe::CpeProcessor;
use processors::cve::CveProcessor;
use status_tracker::SourceUpdateStatusTracker;

use vulnupdater::{Datasource, VulnUpdater};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    tracing_subscriber::fmt::init();

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
