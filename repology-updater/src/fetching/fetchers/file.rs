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

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/data.txt")
            .with_status(200)
            .with_body("Success")
            .create();

        let tmpdir = tempfile::tempdir().unwrap();
        let state_path = tmpdir.path().join("state");

        let fetcher = FileFetcher::new(FileFetcherOptions {
            url: server.url() + "/data.txt",
        });
        let fetch_result = fetcher
            .fetch(&state_path, FetchPoliteness::default())
            .await
            .unwrap();

        mock.assert();
        assert!(fetch_result.was_modified);
        assert_eq!(
            std::fs::read_to_string(fetch_result.state_path.join("data")).unwrap(),
            "Success"
        );
        assert!(fetch_result.accept().await.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_fail() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("GET", "/data.txt").with_status(404).create();

        let tmpdir = tempfile::tempdir().unwrap();
        let state_path = tmpdir.path().join("state");

        let fetcher = FileFetcher::new(FileFetcherOptions {
            url: server.url() + "/data.txt",
        });
        let fetch_result = fetcher.fetch(&state_path, FetchPoliteness::default()).await;

        mock.assert();
        assert!(fetch_result.is_err());
    }
}
