// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use indoc::indoc;

use crate::result::EndpointResult;
use crate::state::AppState;

use super::common::ApiV1Package;

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn api_v1_project(
    Path(project_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
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
