// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;
use std::io::Write;
use std::path::Path;

use futures_util::StreamExt;
use serde::Deserialize;

use crate::fetching::fetcher::{FetchStatus, Fetcher};
use crate::fetching::politeness::FetchPoliteness;
use crate::utils::transact_dir;

const FILE_NAME: &str = "data";

#[derive(Deserialize)]
pub struct FileFetcherOptions {
    pub url: String,
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
            reqwest::get(&self.options.url).await?.bytes_stream()
        };

        let mut file = File::create(&path)?;

        while let Some(item) = stream.next().await {
            file.write_all(&item?)?;
        }

        Ok(FetchStatus {
            was_modified: true,
            state_path: tx.path.to_path_buf(),
            acceptor: Box::new(|| {
                Box::pin(async {
                    tx.commit()?;
                    Ok(())
                })
            }),
        })
    }
}
