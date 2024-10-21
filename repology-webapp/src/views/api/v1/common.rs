// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;
use sqlx::FromRow;

use repology_common::PackageStatus;

#[derive(Serialize, FromRow)]
pub struct ApiV1Package {
    pub repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subrepo: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub srcname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binname: Option<String>,
    pub visiblename: String,

    pub version: String,
    //#[serde(skip_serializing_if = "Option::is_none")]  // Note: this is commented
    // for bug-to-bug compatibility with python webapp
    pub origversion: Option<String>,

    pub status: PackageStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub maintainers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub licenses: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulnerable: Option<bool>,
}
