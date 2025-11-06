// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
#[coverage(off)]
mod tests;

use std::borrow::Cow;
use std::path::Path;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::bail;
use serde::{Deserialize, Serialize};

use crate::fetching::compression::Compression;
use crate::fetching::fetcher::{FetchStatus, Fetcher};
use crate::fetching::http::Http;
use crate::fetching::io::save_http_stream_to_file;
use crate::utils::transact_dir;

use tracing::error;

const STATE_FILE_NAME: &str = "state";
const METADATA_FILE_NAME: &str = "metadata.json";

#[derive(Deserialize)]
#[serde(default)]
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

#[derive(Serialize, Deserialize, Default)]
struct FetchMetadata {
    etag: Option<String>,
}

impl FetchMetadata {
    pub fn read(path: &Path) -> anyhow::Result<Self> {
        Ok(serde_json::from_str::<Self>(&std::fs::read_to_string(
            path,
        )?)?)
    }

    pub fn write(&self, path: &Path) -> anyhow::Result<()> {
        std::fs::write(path, &serde_json::to_string(&self)?)?;
        Ok(())
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
    async fn fetch(&self, path: &Path, http: &Http) -> anyhow::Result<FetchStatus> {
        let mut url = Cow::Borrowed(&self.options.url);
        if let Some(cache_buster) = &self.options.cache_buster {
            let url = url.to_mut();
            *url = url.replace(
                cache_buster,
                &format!(
                    "{}",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                ),
            );
        }

        let dir = transact_dir::TransactionalDir::new(path);
        dir.cleanup()?;

        let current_state = dir.current_state();
        let current_metadata = current_state
            .as_ref()
            .and_then(|state| {
                let path = state.path.join(METADATA_FILE_NAME);
                FetchMetadata::read(&path)
                    .inspect_err(|err| {
                        error!(?err, ?path, "cannot read fetch metadata");
                    })
                    .ok()
            })
            .unwrap_or_default();

        let _permit;
        let response = {
            let client = http.create_client()?;
            let mut request_builder = client.get(&*url);
            if let Some(timeout) = self.options.timeout {
                request_builder = request_builder.timeout(timeout);
            }
            if let Some(etag) = current_metadata.etag.as_ref() {
                request_builder = request_builder.header("if-none-match", etag);
            }

            _permit = http.acquire(&url).await;
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

        let total_size =
            save_http_stream_to_file(response, &next_state_path, self.options.compression).await?;

        if total_size == 0 && !self.options.allow_zero_size {
            bail!("refusing to accept zero size response");
        }

        let next_metadata_path = next_state.path.join(METADATA_FILE_NAME);
        if let Err(err) = new_metadata.write(&next_metadata_path) {
            error!(?err, path = ?next_metadata_path, "cannot write fetch metadata");
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
