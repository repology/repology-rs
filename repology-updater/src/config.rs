// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
#[command(version = None)]
pub enum Commands {
    /// Low level operations
    Raw {
        #[command(subcommand)]
        command: RawCommands,
    },
}

#[derive(Subcommand)]
#[command(version = None)]
pub enum RawCommands {
    /// Just run a parser
    Parse {
        /// Parser name
        #[arg(long = "parser", value_name = "PARSER")]
        parser_name: String,

        /// State path
        #[arg(long, value_name = "PATH")]
        state_path: PathBuf,

        /// Print parsed packages to stdout
        #[arg(long)]
        print: bool,
    },
}
