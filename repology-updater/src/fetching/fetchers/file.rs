// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use serde::Deserialize;

use crate::fetching::fetcher::{Fetcher, FetcherFinalizationHandle};
use crate::utils::transact_dir;

const FILE_NAME: &str = "data";

#[derive(Deserialize)]
pub struct FileFetcherOptions {
    pub url: String,
}

struct FinalizationHandle {
    path: PathBuf,
    handle: transact_dir::WriteHandle,
}

impl FinalizationHandle {
    pub fn new(handle: transact_dir::WriteHandle) -> Self {
        Self {
            path: handle.path.join(FILE_NAME),
            handle,
        }
    }
}

#[async_trait::async_trait]
impl FetcherFinalizationHandle for FinalizationHandle {
    async fn accept(self: Box<Self>) -> anyhow::Result<()> {
        self.handle.commit()?;
        Ok(())
    }

    fn path(&self) -> &Path {
        &self.path
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
    async fn fetch(&self, path: &Path) -> anyhow::Result<Box<dyn FetcherFinalizationHandle>> {
        let dir = transact_dir::TransactionalDir::new(path);
        dir.cleanup()?;
        let tx = dir.begin_replace()?;
        let path = tx.path.join(FILE_NAME);

        let mut stream = reqwest::get(&self.options.url).await?.bytes_stream();

        let mut file = File::create(&path)?;

        while let Some(item) = stream.next().await {
            file.write_all(&item?)?;
        }

        Ok(Box::new(FinalizationHandle::new(tx)))
    }
}
