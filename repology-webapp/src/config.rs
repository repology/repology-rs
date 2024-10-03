// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::net::SocketAddr;
use std::path::PathBuf;

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

    /// Ignore rules tagged with these values
    #[arg(short = 'l', long = "listen", value_name = "ADDR:PORT")]
    pub listen: String,

    /// Path to log directory
    ///
    /// When specified, output is redirected to a log file in the
    /// given directory with daily rotation and 14 kept rotated files.
    #[arg(long, value_name = "PATH")]
    pub log_directory: Option<PathBuf>,

    /// Host/port for Prometheus metrics export endpoint
    #[arg(long, value_name = "HOST:PORT")]
    pub prometheus_export: Option<SocketAddr>,
}
