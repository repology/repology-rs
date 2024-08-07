// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::extract::{Path, State};
use axum::response::Json;
use crate::state::AppState;
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
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[axum::debug_handler]
pub async fn api_v1_project(
    Path(project_name): Path<String>,
    State(state): State<AppState>,
) -> Json<Vec<ApiV1Package>> {
    let packages: Vec<ApiV1Package> = sqlx::query_as(
        r#"
        SELECT
            repo,
            subrepo,
            srcname,
            binname,
            visiblename,
            version,
            CASE WHEN rawversion = version THEN NULL ELSE rawversion END AS origversion,
            versionclass AS status,
            comment AS summary,
            coalesce(maintainers, '{}'::text[]) AS maintainers,
            coalesce(licenses, '{}'::text[]) AS licenses,
            CASE WHEN category IS NULL THEN '{}'::text[] ELSE ARRAY[category] END AS categories,
            NULL AS vulnerable
        FROM packages
        WHERE effname = $1
        "#,
    )
    .bind(project_name)
    .fetch_all(&state.pool)
    .await
    .unwrap();

    Json(packages)
}
