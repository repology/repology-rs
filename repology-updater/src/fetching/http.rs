// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use serde::Deserialize;

use crate::fetching::politeness::FetchPoliteness;

pub struct Http {
    politeness: FetchPoliteness,
    user_agent: String,
}

impl Default for Http {
    fn default() -> Self {
        Self {
            politeness: FetchPoliteness::default(),
            // TODO: make configurable
            user_agent: "repology-fetcher/0 (+https://repology.org/docs/bots)".to_string(),
        }
    }
}

impl Http {
    // TODO: provide fetch methods which handle politeness internally
    // instead of just wrapping it
    pub async fn acquire(&self, url: &str) -> crate::fetching::politeness::HostPermit {
        self.politeness.acquire(url).await
    }

    pub fn create_client(&self) -> Result<reqwest::Client, reqwest::Error> {
        reqwest::Client::builder()
            .user_agent(&self.user_agent)
            .build()
    }

    pub fn start_request(&self) -> RequestWrapper<'_> {
        RequestWrapper::new(self)
    }
}

pub struct RequestWrapper<'a> {
    http: &'a Http,
    timeout: Option<Duration>,
}

impl<'a> RequestWrapper<'a> {
    fn new(http: &'a Http) -> RequestWrapper<'a> {
        Self {
            http,
            timeout: None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> RequestWrapper<'a> {
        self.timeout = Some(timeout);
        self
    }

    pub async fn fetch_text(&self, url: &str) -> anyhow::Result<String> {
        let mut request_builder = self.http.create_client()?.get(url);
        if let Some(timeout) = self.timeout {
            request_builder = request_builder.timeout(timeout);
        }

        let _permit = self.http.politeness.acquire(url).await;
        Ok(request_builder
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?)
    }

    #[expect(unused)] // will be used in fetchers; otherwise, convenient to have for future
    pub async fn fetch_json<T>(&self, url: &str) -> anyhow::Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        Ok(serde_json::from_str(&self.fetch_text(url).await?)?)
    }

    pub async fn fetch_xml<T>(&self, url: &str) -> anyhow::Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        Ok(serde_xml_rs::from_str(&self.fetch_text(url).await?)?)
    }
}
