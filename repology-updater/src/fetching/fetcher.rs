// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

use crate::fetching::http::Http;

pub struct FetchStatus {
    pub was_modified: bool,
    pub state_path: PathBuf,
    // XXX: shouldn't this work?
    //pub(super) acceptor: Box<dyn AsyncFnOnce() -> anyhow::Result<()>>,
    #[allow(clippy::type_complexity)]
    pub(super) acceptor: Box<
        dyn FnOnce() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>,
        >,
    >,
}

impl FetchStatus {
    pub async fn accept(self) -> anyhow::Result<()> {
        (self.acceptor)().await
    }
}

#[async_trait::async_trait]
pub trait Fetcher: Send + Sync {
    async fn fetch(&self, path: &Path, http: &Http) -> anyhow::Result<FetchStatus>;
}
