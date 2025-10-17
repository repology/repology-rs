// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod freebsd;
pub mod stalix;
pub mod tincan;
pub mod yacp;

pub use freebsd::*;
pub use stalix::*;
pub use tincan::*;
pub use yacp::*;

use crate::parsing::parser::PackageParser;

pub fn create_parser(name: &str) -> anyhow::Result<Box<dyn PackageParser>> {
    match name {
        "FreeBsdParser" => Ok(Box::new(freebsd::FreeBsdParser {})),
        "StalIxParser" => Ok(Box::new(stalix::StalIxParser {})),
        "TinCanParser" => Ok(Box::new(tincan::TinCanParser {})),
        "YacpParser" => Ok(Box::new(yacp::YacpParser {})),
        _ => Err(anyhow::anyhow!("invalid parser name {}", name)),
    }
}
