// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
#[coverage(off)]
mod tests;

use std::io;
use std::path::Path;
use std::time::Duration;

use anyhow::bail;
use async_compression::tokio::bufread::{BzDecoder, GzipDecoder, XzDecoder, ZstdDecoder};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;

use crate::fetching::fetcher::{FetchStatus, Fetcher};
use crate::fetching::politeness::FetchPoliteness;
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
enum Compression {
    Gz,
    Xz,
    Bz2,
    Zstd,
}

impl Compression {
    pub fn from_extension(
        path_or_url: &str,
        original_extension: &str,
    ) -> anyhow::Result<Option<Self>> {
        if path_or_url.ends_with(&format!(".{}", original_extension)) {
            Ok(None)
        } else if path_or_url.ends_with(&format!(".{}.gz", original_extension)) {
            Ok(Some(Self::Gz))
        } else if path_or_url.ends_with(&format!(".{}.bz2", original_extension)) {
            Ok(Some(Self::Bz2))
        } else if path_or_url.ends_with(&format!(".{}.xz", original_extension)) {
            Ok(Some(Self::Xz))
        } else if path_or_url.ends_with(&format!(".{}.zst", original_extension)) {
            Ok(Some(Self::Zstd))
        } else {
            bail!(
                "cannot determine compression from file extension {}",
                path_or_url
            );
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct RepodataFetcherOptions {
    pub url: String,
    pub timeout: Option<Duration>,
    pub user_agent: String,
}

impl Default for RepodataFetcherOptions {
    fn default() -> Self {
        Self {
            url: String::new(),
            timeout: Some(Duration::from_mins(1)),
            user_agent: "repology-fetcher/0 (+https://repology.org/docs/bots)".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
struct FetchMetadata {
    checksum: Option<String>,
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

impl RepodataFetcher {
    async fn fetch_repo_md(
        &self,
        url: &str,
        politeness: &FetchPoliteness,
    ) -> anyhow::Result<data::RepoMd> {
        let client = reqwest::Client::builder()
            .user_agent(&self.options.user_agent)
            .build()?;
        let mut request_builder = client.get(url);
        if let Some(timeout) = self.options.timeout {
            request_builder = request_builder.timeout(timeout);
        }

        let _permit = politeness.acquire(url);
        let xml = request_builder
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(serde_xml_rs::from_str(&xml)?)
    }
}

#[async_trait::async_trait]
impl Fetcher for RepodataFetcher {
    async fn fetch(&self, path: &Path, politeness: FetchPoliteness) -> anyhow::Result<FetchStatus> {
        let dir = transact_dir::TransactionalDir::new(path);
        dir.cleanup()?;

        let current_state = dir.current_state();

        let repo_md_url = format!("{}repodata/repomd.xml", self.options.url);
        let repo_md_data = self
            .fetch_repo_md(&repo_md_url, &politeness)
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
                    state_path: current_state.path.to_path_buf().join(STATE_FILE_NAME),
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
        };

        let next_state = dir.begin_replace()?;
        let next_state_path = next_state.path.join(STATE_FILE_NAME);

        let primary_url = format!("{}{}", self.options.url, repo_md_data.location.href);

        let response = {
            let client = reqwest::Client::builder()
                .user_agent(&self.options.user_agent)
                .build()?;
            let mut request_builder = client.get(&primary_url);
            if let Some(timeout) = self.options.timeout {
                request_builder = request_builder.timeout(timeout);
            }

            let _permit = politeness.acquire(&primary_url);
            request_builder.send().await?.error_for_status()?
        };

        let mut file = File::create(&next_state_path).await?;

        let stream = response.bytes_stream();
        let reader = StreamReader::new(stream.map(|r| r.map_err(io::Error::other)));

        let mut decoder: Box<dyn AsyncRead + Unpin + Send> =
            match Compression::from_extension(&primary_url, "xml")? {
                None => Box::new(reader),
                Some(Compression::Gz) => Box::new(GzipDecoder::new(reader)),
                Some(Compression::Xz) => Box::new(XzDecoder::new(reader)),
                Some(Compression::Bz2) => Box::new(BzDecoder::new(reader)),
                Some(Compression::Zstd) => Box::new(ZstdDecoder::new(reader)),
            };

        tokio::io::copy(&mut decoder, &mut file).await?;

        file.sync_all().await?;

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
