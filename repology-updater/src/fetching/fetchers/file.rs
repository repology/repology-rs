// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
//#[coverage(off)]
mod tests;

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use futures_util::StreamExt;
use serde::Deserialize;

use crate::fetching::fetcher::{FetchStatus, Fetcher};
use crate::fetching::politeness::FetchPoliteness;
use crate::utils::transact_dir;

const FILE_NAME: &str = "data";

#[derive(Deserialize)]
pub enum Compression {
    Gz,
    Xz,
    Bz2,
    Zstd,
}

#[derive(Deserialize)]
pub struct FileFetcherOptions {
    pub url: String,
    pub compression: Option<Compression>,
    pub timeout: Option<Duration>,
    pub allow_zero_size: bool,
    pub cache_buster: Option<String>,
}

impl Default for FileFetcherOptions {
    fn default() -> Self {
        Self {
            url: String::new(),
            compression: None,
            timeout: Some(Duration::from_mins(1)),
            allow_zero_size: true,
            cache_buster: None,
        }
    }
}

pub struct FileFetcher {
    options: FileFetcherOptions,
}

impl FileFetcher {
    pub fn new(options: FileFetcherOptions) -> Self {
        Self { options }
    }
}

#[async_trait::async_trait]
impl Fetcher for FileFetcher {
    async fn fetch(&self, path: &Path, politeness: FetchPoliteness) -> anyhow::Result<FetchStatus> {
        let dir = transact_dir::TransactionalDir::new(path);
        dir.cleanup()?;
        let tx = dir.begin_replace()?;
        let path = tx.path.join(FILE_NAME);

        let mut stream = {
            let _permit = politeness.acquire(&self.options.url);
            reqwest::get(&self.options.url)
                .await?
                .error_for_status()?
                .bytes_stream()
        };

        let mut file = File::create(&path)?;

        while let Some(item) = stream.next().await {
            file.write_all(&item?)?;
        }

        Ok(FetchStatus {
            was_modified: true,
            state_path: tx.path.to_path_buf().join(FILE_NAME),
            acceptor: Box::new(|| {
                Box::pin(async {
                    tx.commit()?;
                    Ok(())
                })
            }),
        })
    }
}
