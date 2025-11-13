// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod freebsd;
mod repodata;
mod stalix;
mod tincan;
mod yacp;

use crate::parsing::parser::PackageParser;

pub fn create_parser_options_yaml(
    name: &str,
    options_yaml: &str,
) -> anyhow::Result<Box<dyn PackageParser>> {
    match name {
        "FreeBsdParser" => Ok(Box::new(freebsd::FreeBsdParser {})),
        "StalIxParser" => Ok(Box::new(stalix::StalIxParser {})),
        "TinCanParser" => Ok(Box::new(tincan::TinCanParser {})),
        "YacpParser" => Ok(Box::new(yacp::YacpParser {})),
        "RepodataParser" => {
            let options: repodata::RepodataParserOptions = serde_saphyr::from_str(options_yaml)?;
            Ok(Box::new(repodata::RepodataParser::new(options)))
        }
        _ => Err(anyhow::anyhow!("invalid parser name {}", name)),
    }
}
