// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::extract::{Path, State};
use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;
use indoc::indoc;
use metrics::counter;
use serde::Serialize;
use sqlx::FromRow;

use repology_common::PackageStatus;

use crate::result::EndpointResult;
use crate::state::AppState;

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

pub async fn api_v1_project(
    Path(project_name): Path<String>,
    State(state): State<AppState>,
) -> EndpointResult {
    counter!("repology_webapp.endpoints.requests_total", "endpoint" => "api_v1_project")
        .increment(1);

    let packages: Vec<ApiV1Package> = sqlx::query_as(indoc! {"
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
            NULLIF((flags & (1 << 16))::boolean, false) AS vulnerable
        FROM packages
        WHERE effname = $1
    "})
    .bind(project_name)
    .fetch_all(&state.pool)
    .await?;

    let body = serde_json::to_string(&packages)?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        )],
        body,
    )
        .into_response())
}
