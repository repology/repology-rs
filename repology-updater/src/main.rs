// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(file_buffered)]
#![feature(coverage_attribute)]

mod config;
mod raw_commands;

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
