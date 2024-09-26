// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// PostgreSQL database DSN
    #[arg(short = 'd', long, value_name = "DSN")]
    pub dsn: String,

    /// Enable fast (realtime) CVE feed update
    #[arg(short = 'f', long)]
    pub update_fast_feed: bool,

    /// Enable slow (yearly) CVE feed updates
    #[arg(short = 's', long)]
    pub update_slow_feeds: bool,

    /// Enable CPE dictionary update
    #[arg(short = 'c', long)]
    pub update_cpe_dictionary: bool,

    /// Run one update cycle then exit (otherwise update continously)
    #[arg(short = '1', long)]
    pub once_only: bool,
}
