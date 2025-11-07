// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(assert_matches)]
#![feature(const_trait_impl)]
#![feature(coverage_attribute)]
#![feature(debug_closure_helpers)]
#![feature(file_buffered)]
#![feature(iter_collect_into)]
#![feature(test)]
#![feature(trait_alias)]

mod config;
mod fetching;
mod package;
mod parsing;
mod raw_commands;
mod repositories_config;
mod ruleset;
mod utils;

use clap::Parser;

use config::{CliArgs, Commands};

fn main() {
    let args = CliArgs::parse();

    match &args.command {
        Commands::Raw { command } => {
            raw_commands::raw_command(command, &args);
        }
    }
}
