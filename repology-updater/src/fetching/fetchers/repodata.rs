// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
#[coverage(off)]
mod tests;

use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use crate::fetching::compression::Compression;
use crate::fetching::fetcher::{FetchStatus, Fetcher};
use crate::fetching::http::Http;
use crate::fetching::io::save_http_stream_to_file;
use crate::fetching::metadata::FetchMetadata;
use crate::utils::transact_dir;

use tracing::error;

#[allow(unused)]
mod data {
    use anyhow::bail;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Checksum {
        #[serde(rename = "@type")]
        pub r#type: String,
        #[serde(rename = "#text")]
        pub value: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Location {
        #[serde(rename = "@href")]
        pub href: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Data {
        #[serde(rename = "@type")]
        pub r#type: String,
        pub checksum: Checksum,
        #[serde(rename = "open-checksum")]
        pub open_checksum: Checksum,
        pub location: Location,
        pub timestamp: u64,
        pub size: u64,
        #[serde(rename = "open-size")]
        pub open_size: u64,
    }

    impl Data {
        pub fn checksum(&self) -> &str {
            &self.open_checksum.value
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct RepoMd {
        pub revision: u64,
        #[serde(rename = "data")]
        pub datas: Vec<Data>,
    }

    impl RepoMd {
        pub fn into_data(self) -> anyhow::Result<Data> {
            const REQUIRED_TYPE: &str = "primary";
            for data in self.datas {
                if data.r#type == REQUIRED_TYPE {
                    return Ok(data);
                }
            }
            bail!(
                "cannot find required <data> entry with type=\"{}\"> in repomd.xml",
                REQUIRED_TYPE
            );
        }
    }
}

const STATE_FILE_NAME: &str = "state";
const METADATA_FILE_NAME: &str = "metadata.json";

#[derive(Deserialize)]
#[serde(default)]
pub struct RepodataFetcherOptions {
    pub url: String,
    pub timeout: Duration,
}

impl Default for RepodataFetcherOptions {
    fn default() -> Self {
        Self {
            url: String::new(),
            timeout: Duration::from_mins(1),
        }
    }
}

pub struct RepodataFetcher {
    options: RepodataFetcherOptions,
}

impl RepodataFetcher {
    pub fn new(mut options: RepodataFetcherOptions) -> Self {
        if !options.url.ends_with("/") {
            options.url += "/";
        }
        Self { options }
    }
}

#[async_trait::async_trait]
impl Fetcher for RepodataFetcher {
    async fn fetch(&self, path: &Path, http: &Http) -> anyhow::Result<FetchStatus> {
        let dir = transact_dir::TransactionalDir::new(path);
        dir.cleanup()?;

        let current_state = dir.current_state();

        let repo_md_data = http
            .start_request()
            .with_timeout(self.options.timeout)
            .fetch_xml::<data::RepoMd>(&format!("{}repodata/repomd.xml", self.options.url))
            .await?
            .into_data()?;

        if let Some(current_state) = current_state {
            let current_metadata_path = current_state.path.join(METADATA_FILE_NAME);
            let metadata_checksum = FetchMetadata::read(&current_metadata_path)
                .inspect_err(|err| {
                    error!(?err, path = ?current_metadata_path, "cannot read fetch metadata");
                })
                .unwrap_or_default()
                .checksum;

            if metadata_checksum
                .is_some_and(|metadata_checksum| metadata_checksum == repo_md_data.checksum())
            {
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
        }

        let new_metadata = FetchMetadata {
            checksum: Some(repo_md_data.checksum().into()),
            ..Default::default()
        };

        let next_state = dir.begin_replace()?;
        let next_state_path = next_state.path.join(STATE_FILE_NAME);

        let primary_url = format!("{}{}", self.options.url, repo_md_data.location.href);

        let _permit = http.acquire(&primary_url).await;
        let response = http
            .create_client()?
            .get(&primary_url)
            .timeout(self.options.timeout)
            .send()
            .await?
            .error_for_status()?;

        save_http_stream_to_file(
            response,
            &next_state_path,
            Compression::from_extension(&primary_url, ".xml")?,
        )
        .await?;

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
