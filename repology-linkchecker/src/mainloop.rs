// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::PgPool;
use tracing::info;

use crate::Config;
use crate::delayer::Delayer;
use crate::feeder::Feeder;
use crate::hosts::Hosts;
use crate::http_client::native::NativeHttpClient;
use crate::http_client::python::PythonHttpClient;
use crate::queuer::Queuer;
use crate::resolver::Resolver;
use crate::updater::Updater;

pub async fn link_check_loop(pool: PgPool, config: Config) -> Result<()> {
    let user_agent = format!(
        "repology-linkchecker/1 (+{}/docs/bots)",
        config.repology_host
    );
    let mut feeder = Feeder::new(pool.clone())
        .with_batch_size(config.batch_size)
        .with_batch_period(config.batch_period)
        .with_database_retry_period(config.database_retry_period);
    let http_client = PythonHttpClient::new(&user_agent, &config.python_path)
        .await
        .with_context(|| "failed to initialize http client")?;
    let experimental_http_client = NativeHttpClient::new(user_agent.to_string());
    let resolver = Resolver::new();
    let updater = Updater::new(pool)
        .with_dry_run(config.dry_run)
        .with_database_retry_period(config.database_retry_period);
    let hosts = Hosts::new(
        config.default_host_settings.clone(),
        config.host_settings.clone(),
    );
    let delayer = Delayer::new();
    let queuer = Queuer::new(
        resolver,
        hosts,
        delayer,
        http_client,
        experimental_http_client,
        updater,
    )
    .with_max_queued_urls(config.max_queued_urls)
    .with_max_queued_urls_per_bucket(config.max_queued_urls_per_bucket)
    .with_max_buckets(config.max_buckets)
    .with_disable_ipv4(config.disable_ipv4)
    .with_disable_ipv6(config.disable_ipv6)
    .with_satisfy_with_ipv6(config.satisfy_with_ipv6);

    loop {
        let tasks = feeder.get_next_batch().await;

        if tasks.is_empty() {
            info!("loop over database complete");
            if config.once_only {
                info!("finishing due to --once-only mode");
                return Ok(());
            } else {
                tokio::time::sleep(Duration::from_mins(1)).await;
            }
        }

        for task in tasks {
            queuer.try_put(task).await;
        }
    }
}
