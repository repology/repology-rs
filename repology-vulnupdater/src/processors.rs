// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod cpe;
pub mod cve;

use anyhow::Error;
use async_trait::async_trait;

pub struct DatasourceProcessStatus {
    pub num_changes: u64,
}

#[async_trait]
pub trait DatasourceProcessor {
    async fn process(&self, data: &str) -> Result<DatasourceProcessStatus, Error>;
    async fn finalize(&self) -> Result<(), Error>;
}
