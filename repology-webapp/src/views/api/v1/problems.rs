// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use indoc::indoc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::result::EndpointResult;
use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(rename = "start")]
    pub start_project_name: Option<String>,
}

#[derive(FromRow, Serialize)]
struct ApiV1Problem {
    r#type: String,
    data: serde_json::Value,
    project_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    maintainer: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    binname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    srcname: Option<String>,
    version: String,
    rawversion: String,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all, fields(repository_name = repository_name)))]
pub async fn api_v1_repository_problems(
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    if !state
        .repository_data_cache
        .snapshot()
        .is_repository_active(&repository_name)
    {
        return Ok((StatusCode::NOT_FOUND, "repository not found".to_owned()).into_response());
    }

    let problems: Vec<ApiV1Problem> = sqlx::query_as(indoc! {r#"
        SELECT
            "type"::text AS "type",
            data,
            problems.effname AS project_name,
            problems.maintainer AS maintainer,

            packages.srcname AS srcname,
            packages.binname AS binname,
            packages.version AS version,
            packages.rawversion AS rawversion
        FROM problems
        INNER JOIN packages ON packages.id = problems.package_id
        WHERE
            problems.repo = $1
            AND ($2 IS NULL OR problems.effname >= $2)
        ORDER BY
            problems.effname, problems.maintainer, type
        LIMIT $3
    "#})
    .bind(&repository_name)
    .bind(&query.start_project_name)
    .bind(crate::constants::API_PROBLEMS_PER_PAGE as i32)
    .fetch_all(&state.pool)
    .await?;

    let body = serde_json::to_string(&problems)?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        )],
        body,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all, fields(maintainer_name = maintainer_name, repository_name = repository_name)))]
pub async fn api_v1_maintainer_problems(
    Path((maintainer_name, repository_name)): Path<(String, String)>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    if !state
        .repository_data_cache
        .snapshot()
        .is_repository_active(&repository_name)
    {
        return Ok((StatusCode::NOT_FOUND, "repository not found".to_owned()).into_response());
    }

    let problems: Vec<ApiV1Problem> = sqlx::query_as(indoc! {r#"
        SELECT
            "type"::text AS "type",
            data,
            problems.effname AS project_name,
            problems.maintainer AS maintainer,

            packages.srcname AS srcname,
            packages.binname AS binname,
            packages.version AS version,
            packages.rawversion AS rawversion
        FROM problems
        INNER JOIN packages ON packages.id = problems.package_id
        WHERE
            problems.repo = $1
            AND problems.maintainer = $2
            AND ($3 IS NULL OR problems.effname >= $3)
        ORDER BY
            problems.effname, problems.maintainer, type
        LIMIT $4
    "#})
    .bind(&repository_name)
    .bind(&maintainer_name)
    .bind(&query.start_project_name)
    .bind(crate::constants::API_PROBLEMS_PER_PAGE as i32)
    .fetch_all(&state.pool)
    .await?;

    let body = serde_json::to_string(&problems)?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        )],
        body,
    )
        .into_response())
}
