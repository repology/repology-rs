// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Error;
use async_trait::async_trait;

use crate::processors::{DatasourceProcessor, DatasourceUpdateResult};

pub struct CpeDictProcessor {}

#[async_trait]
impl DatasourceProcessor for CpeDictProcessor {
    async fn process(
        &self,
        mut _data: Box<dyn tokio::io::AsyncRead + Send + Unpin>,
    ) -> Result<DatasourceUpdateResult, Error> {
        unimplemented!("CPE dict handling not implemented");
    }
}
