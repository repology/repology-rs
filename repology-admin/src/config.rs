// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// PostgreSQL database DSN
    #[arg(
        short = 'd',
        long = "dsn",
        value_name = "DSN",
        default_value = "postgresql://repology@localhost/repology"
    )]
    pub dsn: String,

    /// Socket address for serving the webapp
    #[arg(short = 'l', long = "listen", value_name = "ADDR:PORT")]
    pub listen: String,

    /// Path to log directory
    ///
    /// When specified, output is redirected to a log file in the
    /// given directory with daily rotation and 14 kept rotated files.
    #[arg(long, value_name = "PATH")]
    pub log_directory: Option<PathBuf>,

    /// Socket address for serving Prometheus metrics
    #[arg(long, value_name = "ADDR:PORT")]
    pub prometheus_export: Option<SocketAddr>,

    /// Repology host to link to
    #[arg(long, value_name = "HOST", default_value = "https://repology.org")]
    pub repology_host: String,

    /// Allow to do any changes to the database
    ///
    /// The application does not implement any auth mechanisms and
    /// is intended to be served through reverse proxy which takes
    /// care of HTTP auth. This flag is a safety measure which enables
    /// changing the database only after properly set up deploy.
    #[arg(long)]
    pub allow_changes: bool,
}

impl Config {
    pub fn parse() -> Result<Self> {
        let mut config =
            Self::try_parse().with_context(|| "cannot parse command line arguments")?;
        while config.repology_host.ends_with("/") {
            config.repology_host.pop();
        }
        Ok(config)
    }
}
