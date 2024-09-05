// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// PostgreSQL database DSN
    #[arg(short = 'd', long = "dsn", value_name = "DSN")]
    pub dsn: String,

    /// Ignore rules tagged with these values
    #[arg(short = 'l', long = "listen", value_name = "ADDR:PORT")]
    pub listen: String,
}
