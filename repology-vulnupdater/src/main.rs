// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(duration_constructors)]
#![feature(try_blocks)]
#![feature(iterator_try_collect)]

mod args;
mod cpe;
mod datasources;
mod processors;
mod update;

use anyhow::{Context, Error};
use clap::Parser;
use sqlx::PgPool;
use std::time::Duration;

use args::Args;
use datasources::generate_datasources;
use processors::DatasourceUpdateResult;
use update::Updater;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    let pool = PgPool::connect(&args.dsn)
        .await
        .context("error creating PostgreSQL connection pool")?;

    let updater = Updater::new(pool.clone()).context("error creating updater")?;

    loop {
        let mut to_next_update = Duration::from_hours(1);

        for datasource in generate_datasources(&args, pool.clone()) {
            let sleep_duration = match updater.update_source(&datasource).await {
                Ok(res) => match res {
                    DatasourceUpdateResult::NoUpdateNeededFor(dur) => dur,
                    _ => datasource.update_period,
                },
                Err(_) => {
                    const RETRY_TIMEOUT: Duration = Duration::from_mins(5);
                    RETRY_TIMEOUT
                }
            };
            to_next_update = to_next_update.min(sleep_duration);
        }

        if args.once_only {
            return Ok(());
        }

        eprintln!("sleeping for {:?} before next iteration", to_next_update);
        tokio::time::sleep(to_next_update).await;
    }
}
