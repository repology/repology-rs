// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use chrono::{TimeDelta, Utc};
use metrics::counter;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::datetime::parse_utc_datetime;

const USER_AGENT: &str = "repology-vulnupdater/1 (+https://repology.org/docs/bots)";
const TIMEOUT: Duration = Duration::from_secs(60);

// According to https://nvd.nist.gov/developers/start-here#divRateLimits
const WAIT_TIME: Duration = Duration::from_secs(6 + 1);

// We depend on absolute time for incremental updates, so it must be in sync
const MAX_TIME_OFFSET: TimeDelta = TimeDelta::minutes(1);

struct Inner {
    client: reqwest::Client,
    last_request_time: Option<Instant>,
}

pub struct NvdFetcher {
    inner: Mutex<Inner>,
}

impl NvdFetcher {
    pub fn new() -> Result<Self> {
        let inner = Inner {
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .connect_timeout(TIMEOUT)
                .read_timeout(TIMEOUT)
                .build()?,
            last_request_time: None,
        };
        Ok(Self {
            inner: Mutex::new(inner),
        })
    }

    async fn fetch(&self, url: &str) -> Result<String> {
        let mut inner = self.inner.lock().await;

        if let Some(elapsed) = inner.last_request_time.map(|instant| instant.elapsed())
            && elapsed < WAIT_TIME
        {
            tokio::time::sleep(WAIT_TIME - elapsed).await;
        }

        inner.last_request_time = Some(Instant::now());

        let response = inner.client.get(url).send().await?;
        let response = response.error_for_status()?;

        Ok(response.text().await?)
    }

    pub fn paginate(&self, url: &str) -> Paginator {
        Paginator {
            fetcher: self,
            url: url.into(),
            start_index: 0,
            total_results: None,
        }
    }
}

#[derive(Deserialize, Default, Clone)]
pub struct Pagination {
    #[serde(rename = "resultsPerPage")]
    pub results_per_page: u64,
    #[serde(rename = "startIndex")]
    pub start_index: u64,
    #[serde(rename = "totalResults")]
    pub total_results: u64,
    pub timestamp: String,
}

pub struct Paginator<'a> {
    fetcher: &'a NvdFetcher,
    url: String,
    start_index: u64,
    total_results: Option<u64>,
}

impl<'a> Paginator<'a> {
    fn construct_page_url(&'a self) -> Cow<'a, str> {
        if self.start_index > 0 {
            Cow::from(format!(
                "{}{}startIndex={}",
                self.url,
                if self.url.contains('?') { '&' } else { '?' },
                self.start_index
            ))
        } else {
            Cow::from(&self.url)
        }
    }

    pub async fn fetch_next(&mut self) -> Result<Option<String>> {
        if let Some(total_results) = self.total_results
            && self.start_index >= total_results
        {
            return Ok(None);
        }

        counter!("repology_vulnupdater_fetcher_requests_total").increment(1);
        let text = self
            .fetcher
            .fetch(&self.construct_page_url())
            .await
            .inspect_err(|_| {
                counter!("repology_vulnupdater_fetcher_requests_failed_total").increment(1)
            })?;
        let pagination: Pagination = serde_json::from_str(&text)?;
        let server_time = parse_utc_datetime(&pagination.timestamp)?;
        let client_time = Utc::now();
        let time_offset = (server_time - client_time).abs();
        if time_offset > MAX_TIME_OFFSET {
            bail!(
                "too big time offset ({}) between client ({:?}) and server ({:?})",
                time_offset,
                client_time,
                server_time,
            );
        }
        self.start_index += pagination.results_per_page;
        self.total_results = Some(pagination.total_results);

        Ok(if pagination.start_index >= pagination.total_results {
            None
        } else {
            Some(text)
        })
    }

    pub fn seek(&mut self, start_index: u64) {
        self.start_index = start_index
    }

    pub fn current_offset(&self) -> u64 {
        self.start_index
    }

    pub fn total_results(&self) -> Option<u64> {
        self.total_results
    }
}
