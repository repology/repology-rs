// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use url::Url;

use crate::hosts::HostSettings;

const DEFAULT_DSN: &str = "postgresql://repology@localhost/repology";
const DEFAULT_REPOLOGY_HOST: &str = "https://repology.org";

const BUILTIN_HOSTS_CONFIG: &str = include_str!("../hosts.toml");

// feeder
pub const DEFAULT_BATCH_SIZE: usize = 1000;
pub const DEFAULT_BATCH_PERIOD: Duration = Duration::from_secs(60);
pub const DEFAULT_DATABASE_RETRY_PERIOD: Duration = Duration::from_secs(60);

// queuer
pub const DEFAULT_MAX_QUEUED_URLS: usize = 100000;
pub const DEFAULT_MAX_QUEUED_URLS_PER_BUCKET: usize = 1000;
pub const DEFAULT_MAX_BUCKETS: usize = 1000;

// Note: do not use default values for args which are also present in
// FileConfig, otherwise config settings will always be overwritten
// by default clap value. Also, since clap does not allow to provide
// default values but not use them, we have to fill default values
// in options docs manually,
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Path to configuration file with default and/or additional settings
    #[arg(short = 'c', long, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Don't write any updates to the database
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Only do one pass over links, do not loop
    #[arg(short = '1', long)]
    pub once_only: bool,

    /// Disable embedded hosts config
    #[arg(long)]
    pub disable_builtin_hosts_config: bool,

    /// Socket address for serving Prometheus metrics
    #[arg(long, value_name = "ADDR:PORT", help_heading = "Monitoring")]
    pub prometheus_export: Option<SocketAddr>,

    /// Path to log directory
    ///
    /// When specified, output is redirected to a log file in the
    /// given directory with daily rotation and 14 kept rotated files.
    #[arg(long, value_name = "PATH", help_heading = "Logging")]
    pub log_directory: Option<PathBuf>,

    /// Loki log collector URL
    #[arg(long, value_name = "URL", help_heading = "Logging")]
    pub loki_url: Option<Url>,

    /// PostgreSQL database DSN
    ///
    /// Default: postgresql://repology@localhost/repology
    #[arg(
        short = 'd',
        long = "dsn",
        value_name = "DSN",
        help_heading = "Database"
    )]
    pub dsn: Option<String>,

    /// Retry period for database failures
    #[arg(long, value_name = "SECONDS", help_heading = "Database")]
    pub database_retry_period: Option<u64>,

    /// Limit on number of parallel link status updates
    #[arg(long, value_name = "COUNT", help_heading = "Database")]
    pub max_parallel_updates: Option<usize>,

    /// Input batch size
    ///
    /// Default: 1000
    #[arg(long, value_name = "COUNT", help_heading = "Check task generation")]
    pub batch_size: Option<usize>,

    /// Input batch period in seconds
    ///
    /// Default: 60
    #[arg(long, value_name = "SECONDS", help_heading = "Check task generation")]
    pub batch_period: Option<u64>,

    /// Maximum number of urls queued for processing
    ///
    /// When this limit is reached, requesting new urls is suspended
    /// until some queued urls are processed.
    ///
    /// Default: 100000
    #[arg(long, value_name = "COUNT", help_heading = "Task queueing")]
    pub max_queued_urls: Option<usize>,

    /// Maximum number of urls queued for processing for a single host
    ///
    /// When this limit is reached, more urls for this host are silently
    /// dropped, with the indent to be processed in the next cycle(s).
    ///
    /// Default: 1000
    #[arg(long, value_name = "COUNT", help_heading = "Task queueing")]
    pub max_queued_urls_per_bucket: Option<usize>,

    /// Maximum host buckets
    ///
    /// Number of per-host url processing queues. When this limit is
    /// reached, requesting new urls is suspended until some queued urls
    /// are processed.
    ///
    /// Default: 1000
    #[arg(long, value_name = "COUNT", help_heading = "Task queueing")]
    pub max_buckets: Option<usize>,

    /// Repology hostname
    ///
    /// This is used in User-Agent HTTP header used by the linkchecker
    /// to provide link to Repology's /docs/bots page.
    ///
    /// Default: https://repology.org
    #[arg(long, value_name = "HOST", help_heading = "HTTP requests")]
    pub repology_host: Option<String>,

    /// Omit IPv4 check
    #[arg(long, help_heading = "Checker behavior")]
    pub disable_ipv4: bool,

    /// Omit IPV6 check
    #[arg(long, help_heading = "Checker behavior")]
    pub disable_ipv6: bool,

    /// Omit IPv4 check if IPv6 check succeeds
    #[arg(long, help_heading = "Checker behavior")]
    pub satisfy_with_ipv6: bool,

    /// Do faster rechecks for failed links
    #[arg(long, help_heading = "Checker behavior")]
    pub fast_failure_recheck: bool,
}

#[derive(Deserialize)]
struct HostSettingsPatch {
    delay: Option<f32>,
    timeout: Option<f32>,
    recheck_manual: Option<f32>,
    recheck_generated: Option<f32>,
    recheck_unsampled: Option<f32>,
    recheck_splay: Option<f32>,
    skip: Option<bool>,
    aggregate: Option<bool>,
    blacklist: Option<bool>,
    hijacked: Option<bool>,
    disable_ipv6: Option<bool>,
    disable_head: Option<bool>,
    monitor: Option<bool>,
    generated_sampling_percentage: Option<u8>,
    is: Option<String>,
}

impl HostSettingsPatch {
    fn check(&self) {
        assert!(
            self.is.is_none()
                || self.delay.is_none()
                    && self.timeout.is_none()
                    && self.recheck_manual.is_none()
                    && self.recheck_generated.is_none()
                    && self.recheck_unsampled.is_none()
                    && self.recheck_splay.is_none()
                    && self.skip.is_none()
                    && self.aggregate.is_none()
                    && self.blacklist.is_none()
                    && self.hijacked.is_none()
                    && self.disable_ipv6.is_none()
                    && self.disable_head.is_none()
                    && self.monitor.is_none()
                    && self.generated_sampling_percentage.is_none(),
            "you can't specify any other settings for host with .is"
        );
    }
}

impl HostSettings {
    fn merge(&mut self, patch: HostSettingsPatch) {
        self.delay = patch
            .delay
            .map(Duration::from_secs_f32)
            .unwrap_or(self.delay);
        self.timeout = patch
            .timeout
            .map(Duration::from_secs_f32)
            .unwrap_or(self.timeout);
        self.recheck_manual = patch
            .recheck_manual
            .map(|days| Duration::from_secs_f32(days * 86400.0))
            .unwrap_or(self.recheck_manual);
        self.recheck_generated = patch
            .recheck_generated
            .map(|days| Duration::from_secs_f32(days * 86400.0))
            .unwrap_or(self.recheck_generated);
        self.recheck_unsampled = patch
            .recheck_unsampled
            .map(|days| Duration::from_secs_f32(days * 86400.0))
            .unwrap_or(self.recheck_unsampled);
        self.recheck_splay = patch.recheck_splay.unwrap_or(self.recheck_splay);
        self.skip = patch.skip.unwrap_or(self.skip);
        self.aggregate = patch.aggregate.unwrap_or(self.aggregate);
        self.blacklist = patch.blacklist.unwrap_or(self.blacklist);
        self.hijacked = patch.hijacked.unwrap_or(self.hijacked);
        self.disable_ipv6 = patch.disable_ipv6.unwrap_or(self.disable_ipv6);
        self.disable_head = patch.disable_head.unwrap_or(self.disable_head);
        self.monitor = patch.monitor.unwrap_or(self.monitor);
        self.generated_sampling_percentage = patch
            .generated_sampling_percentage
            .unwrap_or(self.generated_sampling_percentage);
        if patch.is.is_some() {
            self.is = patch.is;
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    dsn: Option<String>,
    log_directory: Option<PathBuf>,
    loki_url: Option<Url>,
    prometheus_export: Option<SocketAddr>,
    repology_host: Option<String>,
    hosts: HashMap<String, HostSettingsPatch>,
    dry_run: Option<bool>,
    once_only: Option<bool>,
    batch_size: Option<usize>,
    batch_period: Option<u64>,
    database_retry_period: Option<u64>,
    max_queued_urls: Option<usize>,
    max_queued_urls_per_bucket: Option<usize>,
    max_buckets: Option<usize>,
    disable_ipv4: Option<bool>,
    disable_ipv6: Option<bool>,
    satisfy_with_ipv6: Option<bool>,
    fast_failure_recheck: Option<bool>,
    disable_builtin_hosts_config: Option<bool>,
    max_parallel_updates: Option<usize>,
}

#[derive(Debug)]
pub struct Config {
    pub dsn: String,
    pub log_directory: Option<PathBuf>,
    pub loki_url: Option<Url>,
    pub prometheus_export: Option<SocketAddr>,
    pub repology_host: String,
    pub default_host_settings: HostSettings,
    pub host_settings: HashMap<String, HostSettings>,
    pub dry_run: bool,
    pub once_only: bool,
    pub batch_size: usize,
    pub batch_period: Duration,
    pub database_retry_period: Duration,
    pub max_queued_urls: usize,
    pub max_queued_urls_per_bucket: usize,
    pub max_buckets: usize,
    pub disable_ipv4: bool,
    pub disable_ipv6: bool,
    pub satisfy_with_ipv6: bool,
    pub fast_failure_recheck: bool,
    pub max_parallel_updates: usize,
}

impl Config {
    pub fn parse() -> Result<Self> {
        let args = CliArgs::parse();

        let mut config: FileConfig = if let Some(path) = args.config {
            // XXX: a good case for try block to avoid with_context repetition, but heterogeneous
            // try blocks are currently broken, see https://github.com/rust-lang/rust/issues/149025
            let toml = std::fs::read(&path)
                .with_context(|| format!("cannot read config file {}", path.display()))?;
            let toml = std::str::from_utf8(&toml)
                .with_context(|| format!("cannot parse config file {}", path.display()))?;
            toml::from_str(toml)
                .with_context(|| format!("cannot parse config file {}", path.display()))?
        } else {
            Default::default()
        };

        let dsn = args
            .dsn
            .as_deref()
            .or(config.dsn.as_deref())
            .unwrap_or(DEFAULT_DSN)
            .to_string();
        let repology_host = args
            .repology_host
            .as_deref()
            .or(config.repology_host.as_deref())
            .unwrap_or(DEFAULT_REPOLOGY_HOST)
            .trim_end_matches('/')
            .to_string();

        let disable_builtin_hosts_config = args.disable_builtin_hosts_config
            || config.disable_builtin_hosts_config.unwrap_or(false);

        let mut builtin_hosts = if !disable_builtin_hosts_config {
            toml::from_str::<HashMap<String, HostSettingsPatch>>(BUILTIN_HOSTS_CONFIG)
                .expect("builtin hosts.toml should be parsable")
        } else {
            Default::default()
        };

        let mut default_host_settings = HostSettings::default();
        builtin_hosts
            .remove("default")
            .into_iter()
            .chain(config.hosts.remove("default"))
            .for_each(|patch| {
                patch.check();
                default_host_settings.merge(patch);
            });

        let mut host_settings: HashMap<String, HostSettings> = Default::default();
        builtin_hosts
            .into_iter()
            .chain(config.hosts)
            .for_each(|(hostname, patch)| {
                patch.check();
                host_settings
                    .entry(hostname)
                    .or_insert_with(|| default_host_settings.clone())
                    .merge(patch)
            });

        Ok(Config {
            dsn,
            log_directory: args.log_directory.or(config.log_directory),
            loki_url: args.loki_url.or(config.loki_url),
            prometheus_export: args.prometheus_export.or(config.prometheus_export),
            repology_host,
            default_host_settings,
            host_settings,
            dry_run: args.dry_run || config.dry_run.unwrap_or(false),
            once_only: args.once_only || config.once_only.unwrap_or(false),
            batch_size: args
                .batch_size
                .or(config.batch_size)
                .unwrap_or(DEFAULT_BATCH_SIZE),
            batch_period: args
                .batch_period
                .or(config.batch_period)
                .map(Duration::from_secs)
                .unwrap_or(DEFAULT_BATCH_PERIOD),
            database_retry_period: args
                .database_retry_period
                .or(config.database_retry_period)
                .map(Duration::from_secs)
                .unwrap_or(DEFAULT_DATABASE_RETRY_PERIOD),
            max_queued_urls: args
                .max_queued_urls
                .or(config.max_queued_urls)
                .unwrap_or(DEFAULT_MAX_QUEUED_URLS),
            max_queued_urls_per_bucket: args
                .max_queued_urls_per_bucket
                .or(config.max_queued_urls_per_bucket)
                .unwrap_or(DEFAULT_MAX_QUEUED_URLS_PER_BUCKET),
            max_buckets: args
                .max_buckets
                .or(config.max_buckets)
                .unwrap_or(DEFAULT_MAX_BUCKETS),
            disable_ipv4: args.disable_ipv4 || config.disable_ipv4.unwrap_or(false),
            disable_ipv6: args.disable_ipv6 || config.disable_ipv6.unwrap_or(false),
            satisfy_with_ipv6: args.satisfy_with_ipv6 || config.satisfy_with_ipv6.unwrap_or(false),
            fast_failure_recheck: args.fast_failure_recheck
                || config.fast_failure_recheck.unwrap_or(false),
            max_parallel_updates: args
                .max_parallel_updates
                .or(config.max_parallel_updates)
                .unwrap_or(0),
        })
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_hosts_parsing() {
        assert!(toml::from_str::<HashMap<String, HostSettingsPatch>>(BUILTIN_HOSTS_CONFIG).is_ok());
    }
}
