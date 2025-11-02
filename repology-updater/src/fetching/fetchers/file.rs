// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

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
            ..Default::default()
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
            ..Default::default()
        });
        let fetch_result = fetcher.fetch(&state_path, FetchPoliteness::default()).await;

        mock.assert();
        assert!(fetch_result.is_err());
    }

    async fn fetch_with_compression(data: &[u8], compression: Compression) -> String {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/data")
            .with_status(200)
            .with_body(data)
            .create();

        let tmpdir = tempfile::tempdir().unwrap();
        let state_path = tmpdir.path().join("state");

        let fetcher = FileFetcher::new(FileFetcherOptions {
            url: server.url() + "/data",
            compression: Some(compression),
            ..Default::default()
        });
        let fetch_result = fetcher
            .fetch(&state_path, FetchPoliteness::default())
            .await
            .unwrap();

        std::fs::read_to_string(fetch_result.state_path.join("data")).unwrap()
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_bz2() {
        let data = include_bytes!("file_test_data/data.bz2");
        let result = fetch_with_compression(data, Compression::Bz2);
        assert_eq!(result.await.as_str(), "Success");
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_gz() {
        let data = include_bytes!("file_test_data/data.gz");
        let result = fetch_with_compression(data, Compression::Gz);
        assert_eq!(result.await.as_str(), "Success");
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_xz() {
        let data = include_bytes!("file_test_data/data.xz");
        let result = fetch_with_compression(data, Compression::Xz);
        assert_eq!(result.await.as_str(), "Success");
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_zstd() {
        let data = include_bytes!("file_test_data/data.zst");
        let result = fetch_with_compression(data, Compression::Zstd);
        assert_eq!(result.await.as_str(), "Success");
    }

    #[tokio::test]
    #[ignore]
    async fn test_allow_zero_size() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/data")
            .with_status(200)
            .with_body("")
            .create();

        let tmpdir = tempfile::tempdir().unwrap();
        let state_path = tmpdir.path().join("state");

        let fetcher = FileFetcher::new(FileFetcherOptions {
            url: server.url() + "/data",
            allow_zero_size: false,
            ..Default::default()
        });
        let result = fetcher.fetch(&state_path, FetchPoliteness::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore]
    async fn test_timeout() {
        let mut server = mockito::Server::new_async().await;
        // no future or timeout support in mockito (https://github.com/lipanski/mockito/pull/163),
        // so we have to blocking sleep; hopefully it does not affect other tests much
        server
            .mock("GET", "/data")
            .with_status(200)
            .with_body_from_request(|_| {
                std::thread::sleep(Duration::from_millis(20));
                "success".into()
            })
            .create();

        let tmpdir = tempfile::tempdir().unwrap();
        let state_path = tmpdir.path().join("state");

        let fetcher = FileFetcher::new(FileFetcherOptions {
            url: server.url() + "/data",
            timeout: Some(Duration::from_millis(10)),
            ..Default::default()
        });
        let result = fetcher.fetch(&state_path, FetchPoliteness::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore]
    async fn test_cache_buster() {
        let mut server = mockito::Server::new_async().await;
        // no future or timeout support in mockito (https://github.com/lipanski/mockito/pull/163),
        // so we have to blocking sleep; hopefully it does not affect other tests much
        server
            .mock("GET", "/data")
            .with_status(200)
            .with_body_from_request(|request| request.path_and_query().into())
            .create();

        let tmpdir = tempfile::tempdir().unwrap();
        let state_path = tmpdir.path().join("state");

        let fetcher = FileFetcher::new(FileFetcherOptions {
            url: server.url() + "/data?CACHE_BUSTER",
            cache_buster: Some("CACHE_BUSTER".into()),
            ..Default::default()
        });
        let fetch_result = fetcher
            .fetch(&state_path, FetchPoliteness::default())
            .await
            .unwrap();
        let data1 = std::fs::read_to_string(fetch_result.state_path.join("data")).unwrap();
        fetch_result.accept().await.unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;

        let fetch_result = fetcher
            .fetch(&state_path, FetchPoliteness::default())
            .await
            .unwrap();
        let data2 = std::fs::read_to_string(fetch_result.state_path.join("data")).unwrap();
        fetch_result.accept().await.unwrap();

        assert_ne!(data1, data2);
    }
}
