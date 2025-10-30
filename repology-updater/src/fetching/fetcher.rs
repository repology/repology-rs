// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

#[async_trait::async_trait]
pub trait FetcherFinalizationHandle: Send + Sync {
    async fn accept(self: Box<Self>) -> anyhow::Result<()>;
    fn path(&self) -> &Path;
}

#[async_trait::async_trait]
pub trait Fetcher: Send + Sync {
    async fn fetch(&self, path: &Path) -> anyhow::Result<Box<dyn FetcherFinalizationHandle>>;
}
