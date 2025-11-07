// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct FetchMetadata {
    pub etag: Option<String>,
    pub checksum: Option<String>,
}

impl FetchMetadata {
    pub fn read(path: &Path) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }

    pub fn write(&self, path: &Path) -> anyhow::Result<()> {
        std::fs::write(path, &serde_json::to_string(&self)?)?;
        Ok(())
    }
}
