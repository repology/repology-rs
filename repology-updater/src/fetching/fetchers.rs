// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod file;

pub use file::*;

use crate::fetching::fetcher::Fetcher;

pub fn create_fetcher_options_yaml(
    name: &str,
    options_yaml: &str,
) -> anyhow::Result<Box<dyn Fetcher>> {
    match name {
        "FileFetcher" => {
            let options: FileFetcherOptions = serde_saphyr::from_str(options_yaml)?;
            Ok(Box::new(FileFetcher::new(options)))
        }
        _ => Err(anyhow::anyhow!("invalid fetcher name {}", name)),
    }
}
