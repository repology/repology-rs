// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod types;

use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use ip_network::IpNetwork;

use types::{MyIpNetwork, StaffAfkPeriod};

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct CliArgs {
    /// PostgreSQL database DSN
    #[arg(
        short = 'd',
        long = "dsn",
        value_name = "DSN",
        default_value = "postgresql://repology@localhost/repology"
    )]
    dsn: Option<String>,

    /// Ignore rules tagged with these values
    #[arg(short = 'l', long = "listen", value_name = "ADDR:PORT")]
    listen: Option<String>,

    /// Path to log directory
    ///
    /// When specified, output is redirected to a log file in the
    /// given directory with daily rotation and 14 kept rotated files.
    #[arg(long, value_name = "PATH")]
    log_directory: Option<PathBuf>,

    /// Host/port for Prometheus metrics export endpoint
    #[arg(long, value_name = "HOST:PORT")]
    prometheus_export: Option<SocketAddr>,

    /// Path to configuration file
    #[arg(short = 'c', long, value_name = "PATH")]
    config: Option<PathBuf>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    dsn: Option<String>,

    listen: Option<String>,
    log_directory: Option<PathBuf>,
    prometheus_export: Option<SocketAddr>,

    spam_keywords: Vec<String>,
    spam_networks: Vec<MyIpNetwork>,

    disabled_reports: Vec<String>,
    staff_afk_periods: Vec<StaffAfkPeriod>,
}

#[derive(Debug, Default, Clone)]
pub struct AppConfig {
    pub spam_keywords: Vec<String>,
    pub spam_networks: Vec<IpNetwork>,

    pub disabled_reports: HashSet<String>,
    pub staff_afk_periods: Vec<StaffAfkPeriod>,
}

#[derive(Debug)]
pub struct Config {
    pub dsn: String,
    pub listen: String,
    pub log_directory: Option<PathBuf>,
    pub prometheus_export: Option<SocketAddr>,

    pub app_config: AppConfig,
}

impl Config {
    pub fn parse() -> Result<Self> {
        let args = CliArgs::try_parse().with_context(|| "cannot parse command line arguments")?;

        let config = args
            .config
            .map(|path| {
                let config: Result<FileConfig> = try {
                    toml::from_str::<FileConfig>(std::str::from_utf8(&std::fs::read(&path)?)?)?
                };
                config.with_context(|| format!("cannot parse config file {}", path.display()))
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Config {
            dsn: config
                .dsn
                .or(args.dsn)
                .ok_or_else(|| anyhow!("missing required argument or config paramater \"dsn\""))?,
            listen: config.listen.or(args.listen).ok_or_else(|| {
                anyhow!("missing required argument or config parameter \"listen\"")
            })?,
            log_directory: config.log_directory.or(args.log_directory),
            prometheus_export: config.prometheus_export.or(args.prometheus_export),

            app_config: AppConfig {
                spam_keywords: config.spam_keywords,
                spam_networks: config
                    .spam_networks
                    .into_iter()
                    .map(|network| network.0)
                    .collect(),
                disabled_reports: config.disabled_reports.into_iter().collect(),
                staff_afk_periods: config.staff_afk_periods,
            },
        })
    }
}
