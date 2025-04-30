// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;

use crate::hosts::HostSettings;

const DEFAULT_DSN: &str = "postgresql://repology@localhost/repology";
const DEFAULT_REPOLOGY_HOST: &str = "https://repology.org";

// feeder
pub const DEFAULT_BATCH_SIZE: usize = 1000;
pub const DEFAULT_BATCH_PERIOD: Duration = Duration::from_secs(60);
pub const DEFAULT_DATABASE_RETRY_PERIOD: Duration = Duration::from_secs(60);

// queuer
pub const DEFAULT_MAX_QUEUED_URLS: usize = 100000;
pub const DEFAULT_MAX_QUEUED_URLS_PER_BUCKET: usize = 1000;
pub const DEFAULT_MAX_BUCKETS: usize = 1000;

// requester
pub const DEFAULT_PYTHON_PATH: &str = "/usr/bin/python";

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

    /// Socket address for serving Prometheus metrics
    #[arg(long, value_name = "ADDR:PORT", help_heading = "Monitoring")]
    pub prometheus_export: Option<SocketAddr>,

    /// Path to log directory
    ///
    /// When specified, output is redirected to a log file in the
    /// given directory with daily rotation and 14 kept rotated files.
    #[arg(long, value_name = "PATH", help_heading = "Logging")]
    pub log_directory: Option<PathBuf>,

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

    /// Input batch size
    ///
    /// Default: 10000
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

    /// Path to Python interpreter
    ///
    /// Default: /usr/bin/python
    #[arg(long, value_name = "PATH", help_heading = "HTTP requests")]
    pub python_path: Option<String>,

    /// Repology hostname
    ///
    /// This is used in User-Agent HTTP header used by the linkchecker
    /// to provide link to Repology's /docs/bots page.
    ///
    /// Default: https://repology.org
    #[arg(long, value_name = "HOST", help_heading = "HTTP requests")]
    pub repology_host: Option<String>,

    /// Omit IPv4 check
    #[arg(long, help_heading = "Internet protocol versions")]
    pub disable_ipv4: bool,

    /// Omit IPV6 check
    #[arg(long, help_heading = "Internet protocol versions")]
    pub disable_ipv6: bool,

    /// Omit IPv4 check if IPv6 check succeeds
    #[arg(long, help_heading = "Internet protocol versions")]
    pub satisfy_with_ipv6: bool,
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
    disable_ipv6: Option<bool>,
    disable_head: Option<bool>,
    generated_sampling_percentage: Option<u8>,
}

impl HostSettings {
    fn merge(mut self, patch: HostSettingsPatch) -> Self {
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
        self.disable_ipv6 = patch.disable_ipv6.unwrap_or(self.disable_ipv6);
        self.disable_head = patch.disable_head.unwrap_or(self.disable_head);
        self.generated_sampling_percentage = patch
            .generated_sampling_percentage
            .unwrap_or(self.generated_sampling_percentage);
        self
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    dsn: Option<String>,
    log_directory: Option<PathBuf>,
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
    python_path: Option<String>,
    disable_ipv4: Option<bool>,
    disable_ipv6: Option<bool>,
    satisfy_with_ipv6: Option<bool>,
}

#[derive(Debug)]
pub struct Config {
    pub dsn: String,
    pub log_directory: Option<PathBuf>,
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
    pub python_path: String,
    pub disable_ipv4: bool,
    pub disable_ipv6: bool,
    pub satisfy_with_ipv6: bool,
}

impl Config {
    pub fn parse() -> Result<Self> {
        let args = CliArgs::parse();

        let mut config = args
            .config
            .map(|path| {
                let config: Result<FileConfig> = try {
                    toml::from_str::<FileConfig>(std::str::from_utf8(&std::fs::read(&path)?)?)?
                };
                config.with_context(|| format!("cannot parse config file {}", path.display()))
            })
            .transpose()?
            .unwrap_or_default();

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

        let mut default_host_settings = HostSettings::default();
        if let Some(default_host_settings_patch) = config.hosts.remove("default") {
            default_host_settings = default_host_settings.merge(default_host_settings_patch)
        }

        let host_settings: HashMap<_, _> = config
            .hosts
            .into_iter()
            .map(|(hostname, host_settings_patch)| {
                (
                    hostname,
                    default_host_settings.clone().merge(host_settings_patch),
                )
            })
            .collect();

        Ok(Config {
            dsn,
            log_directory: args.log_directory.or(config.log_directory),
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
            python_path: args
                .python_path
                .or(config.python_path)
                .unwrap_or_else(|| DEFAULT_PYTHON_PATH.to_string()),
            disable_ipv4: args.disable_ipv4 || config.disable_ipv4.unwrap_or(false),
            disable_ipv6: args.disable_ipv6 || config.disable_ipv6.unwrap_or(false),
            satisfy_with_ipv6: args.satisfy_with_ipv6 || config.satisfy_with_ipv6.unwrap_or(false),
        })
    }
}
