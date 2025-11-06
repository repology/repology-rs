// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

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
}
