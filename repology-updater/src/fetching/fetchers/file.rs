// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
//#[coverage(off)]
mod tests;

use std::io;
use std::path::Path;
use std::time::Duration;

use anyhow::bail;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio_util::io::StreamReader;

use crate::fetching::fetcher::{FetchStatus, Fetcher};
use crate::fetching::politeness::FetchPoliteness;
use crate::utils::transact_dir;

use tracing::error;

const STATE_FILE_NAME: &str = "state";
const METADATA_FILE_NAME: &str = "metadata.json";

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
    pub user_agent: String,
}

impl Default for FileFetcherOptions {
    fn default() -> Self {
        Self {
            url: String::new(),
            compression: None,
            timeout: Some(Duration::from_mins(1)),
            allow_zero_size: true,
            cache_buster: None,
            // TODO: make configurable
            user_agent: "repology-fetcher/0 (+https://repology.org/docs/bots)".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FetchMetadata {
    etag: Option<String>,
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

        let current_state = dir.current_state();
        let current_metadata_path = current_state
            .as_ref()
            .map(|state| state.path.join(METADATA_FILE_NAME));

        let current_metadata = current_metadata_path.and_then(|metadata_path| {
            let json = match std::fs::read_to_string(&metadata_path) {
                Ok(json) => json,
                Err(err) => {
                    error!(?err, ?metadata_path, "cannot read fetch metadata");
                    return None;
                }
            };

            match serde_json::from_str::<FetchMetadata>(&json) {
                Ok(metadata) => Some(metadata),
                Err(err) => {
                    error!(?err, ?metadata_path, "cannot parse fetch metadata");
                    None
                }
            }
        });

        let response = {
            let client = reqwest::Client::builder()
                .user_agent(&self.options.user_agent)
                .build()?;
            let mut request_builder = client.get(&self.options.url);
            if let Some(timeout) = self.options.timeout {
                request_builder = request_builder.timeout(timeout);
            }
            if let Some(etag) = current_metadata
                .as_ref()
                .and_then(|metadata| metadata.etag.as_ref())
            {
                request_builder = request_builder.header("if-none-match", etag);
            }

            let _permit = politeness.acquire(&self.options.url);
            request_builder.send().await?.error_for_status()?
        };

        if let Some(current_state) = current_state
            && response.status() == reqwest::StatusCode::NOT_MODIFIED
        {
            return Ok(FetchStatus {
                was_modified: false,
                state_path: current_state.path.to_path_buf().join(STATE_FILE_NAME),
                acceptor: Box::new(|| {
                    Box::pin(async move {
                        let _ = current_state;
                        Ok(())
                    })
                }),
            });
        }

        let new_metadata = FetchMetadata {
            etag: {
                match response.headers().get("etag").map(|etag| etag.to_str()) {
                    Some(Ok(etag)) => Some(etag.to_string()),
                    Some(Err(err)) => {
                        error!(?err, "cannot parse etag header");
                        None
                    }
                    None => None,
                }
            },
        };

        let next_state = dir.begin_replace()?;
        let next_state_path = next_state.path.join(STATE_FILE_NAME);
        let mut file = File::create(&next_state_path).await?;

        let mut stream = response.bytes_stream();
        let mut reader = StreamReader::new(
            stream.map(|r| r.map_err(|e| io::Error::new(io::ErrorKind::Other, e))),
        );

        let total_size = tokio::io::copy(&mut reader, &mut file).await?;

        if total_size == 0 && !self.options.allow_zero_size {
            bail!("refusing to accept zero size response");
        }

        file.sync_all().await?;

        match serde_json::to_string(&new_metadata) {
            Ok(json) => {
                let next_metadata_path = next_state.path.join(METADATA_FILE_NAME);
                if let Err(err) = std::fs::write(&next_metadata_path, json) {
                    error!(?err, ?next_metadata_path, "cannot write fetch metadata");
                }
            }
            Err(err) => {
                error!(?err, "cannot serialize fetch metadata");
            }
        }

        Ok(FetchStatus {
            was_modified: true,
            state_path: next_state_path,
            acceptor: Box::new(|| {
                Box::pin(async move {
                    next_state.commit()?;
                    Ok(())
                })
            }),
        })
    }
}
