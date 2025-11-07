// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Enable debug logging
    #[arg(long)]
    pub debug: bool,

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
    /// Just run a fetcher
    Fetch {
        /// Fetcher name
        #[arg(long = "fetcher", value_name = "PARSER")]
        fetcher_name: String,

        /// State path
        #[arg(long, value_name = "PATH")]
        state_path: PathBuf,

        /// Fetcher options as YAML
        #[arg(long, value_name = "YAML")]
        fetcher_options: String,
    },
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
    /// Run both a fetcher and a parser
    ///
    /// Freshly fetched state is only accepted if parsing succeeds
    FetchParse {
        /// Fetcher name
        #[arg(long = "fetcher", value_name = "PARSER")]
        fetcher_name: String,

        /// Parser name
        #[arg(long = "parser", value_name = "PARSER")]
        parser_name: String,

        /// State path
        #[arg(long, value_name = "PATH")]
        state_path: PathBuf,

        /// Fetcher options as YAML
        #[arg(long, value_name = "YAML")]
        fetcher_options: String,

        /// Print parsed packages to stdout
        #[arg(long)]
        print: bool,
    },
    /// Parse and dump ruleset
    DumpRuleset {
        /// Path to ruleset
        #[arg(long, value_name = "PATH")]
        ruleset_path: PathBuf,
    },
    /// Parse and dump repositories config
    DumpRepositories {
        /// Path to repositories config
        #[arg(long, value_name = "PATH")]
        repositories_path: PathBuf,
    },
}
