// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod cpe_dict;
pub mod cve_feed;

use std::time::Duration;

use anyhow::Error;
use async_trait::async_trait;

#[derive(Debug)]
pub enum DatasourceUpdateResult {
    NoUpdateNeededFor(Duration),
    NoChanges,
    HadChanges(u64),
}

#[async_trait]
pub trait DatasourceProcessor {
    async fn process(
        &self,
        mut data: Box<dyn tokio::io::AsyncRead + Send + Unpin>,
    ) -> Result<DatasourceUpdateResult, Error>;
}
