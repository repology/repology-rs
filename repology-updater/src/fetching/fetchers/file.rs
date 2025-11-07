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
use serde::Deserialize;
use tracing::{error, info};

use crate::fetching::fetcher::{FetchStatus, Fetcher};
use crate::fetching::http::Http;
use crate::fetching::utils::compression::Compression;
use crate::fetching::utils::io::save_http_stream_to_file;
use crate::fetching::utils::metadata::FetchMetadata;
use crate::utils::transact_dir;

const STATE_FILE_NAME: &str = "state";
const METADATA_FILE_NAME: &str = "metadata.json";

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct FileFetcherOptions {
    pub url: String,
    pub compression: Option<Compression>,
    pub timeout: Duration,
    pub allow_zero_size: bool,
    pub cache_buster: Option<String>,
}

impl Default for FileFetcherOptions {
    fn default() -> Self {
        Self {
            url: String::new(),
            compression: None,
            timeout: Duration::from_mins(1),
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
    #[tracing::instrument(name = "FileFetcher", skip_all, fields(url = ?self.options.url))]
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
            let mut request_builder = client.get(&*url).timeout(self.options.timeout);
            if let Some(etag) = current_metadata.etag.as_ref() {
                info!(etag, "adding if-none-match header");
                request_builder = request_builder.header("if-none-match", etag);
            }

            _permit = http.acquire(&url).await;
            request_builder.send().await?.error_for_status()?
        };

        if let Some(current_state) = current_state
            && response.status() == reqwest::StatusCode::NOT_MODIFIED
        {
            info!("server responded with 304 Not Modified");
            return Ok(FetchStatus {
                was_modified: false,
                state_path: current_state.path.join(STATE_FILE_NAME),
                acceptor: Box::new(|| {
                    Box::pin(async move {
                        let _ = current_state;
                        Ok(())
                    })
                }),
            });
        }

        let new_metadata = FetchMetadata {
            etag: match response
                .headers()
                .get("etag")
                .map(|etag| etag.to_str())
                .transpose()?
            {
                Some(etag) => {
                    info!(etag, "got etag value");
                    Some(etag.to_string())
                }
                None => {
                    info!("no etag header in reply");
                    None
                }
            },
            ..Default::default()
        };

        let next_state = dir.begin_replace()?;
        let next_state_path = next_state.path.join(STATE_FILE_NAME);

        let total_size =
            save_http_stream_to_file(response, &next_state_path, self.options.compression).await?;

        if total_size == 0 && !self.options.allow_zero_size {
            bail!("refusing to accept zero size response");
        }

        new_metadata.write(&next_state.path.join(METADATA_FILE_NAME))?;

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
