// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use serde::Deserialize;
use sqlx::FromRow;

use crate::endpoints::Endpoint;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub autorefresh: bool,
}

#[derive(Template)]
#[template(path = "repositories/updates.html")]
struct TemplateParams {
    ctx: TemplateContext,
    repositories: Vec<Repository>,
    autorefresh: bool,
}

#[derive(FromRow)]
struct Repository {
    name: String,
    title: String,

    last_fetch_id: Option<i32>,
    last_fetch_start: Option<DateTime<Utc>>,
    last_fetch_status: Option<String>,
    last_fetch_errors: Option<i32>,
    last_fetch_warnings: Option<i32>,

    last_parse_id: Option<i32>,
    last_parse_start: Option<DateTime<Utc>>,
    last_parse_status: Option<String>,
    last_parse_errors: Option<i32>,
    last_parse_warnings: Option<i32>,

    last_failed_id: Option<i32>,
    last_failed_start: Option<DateTime<Utc>>,
    #[expect(unused)] // TODO: remove
    last_failed_status: Option<String>,
    last_failed_errors: Option<i32>,
    last_failed_warnings: Option<i32>,

    history: Vec<sqlx::types::Json<HistoryItem>>,
}

#[derive(Deserialize)]
struct HistoryItem {
    id: i32,
    status: String,
    r#type: String,
    no_changes: bool,
}

#[cfg_attr(
    not(feature = "coverage"),
    tracing::instrument(skip(gen_path, gen_query, state))
)]
pub async fn repositories_updates(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::RepositoriesUpdates, gen_path, gen_query);

    let repositories: Vec<Repository> = sqlx::query_as(indoc! {r#"
        SELECT
            name,
            "desc" AS title,

            last_fetch_subq.id AS last_fetch_id,
            last_fetch_subq.start_ts AS last_fetch_start,
            last_fetch_subq.status::text AS last_fetch_status,
            last_fetch_subq.num_errors AS last_fetch_errors,
            last_fetch_subq.num_warnings AS last_fetch_warnings,

            last_parse_subq.id AS last_parse_id,
            last_parse_subq.start_ts AS last_parse_start,
            last_parse_subq.status::text AS last_parse_status,
            last_parse_subq.num_errors AS last_parse_errors,
            last_parse_subq.num_warnings AS last_parse_warnings,

            last_failed_subq.id AS last_failed_id,
            last_failed_subq.start_ts AS last_failed_start,
            last_failed_subq.status::text AS last_failed_status,
            last_failed_subq.num_errors AS last_failed_errors,
            last_failed_subq.num_warnings AS last_failed_warnings,

            coalesce((
                SELECT
                    array_agg(json)
                FROM (
                    SELECT
                        json
                    FROM (
                        SELECT
                            start_ts,
                            json_build_object(
                                'id', id,
                                'status', status,
                                'type', "type",
                                'no_changes', no_changes
                            ) AS json
                        FROM
                            runs
                        WHERE repository_id = repositories.id
                        ORDER BY start_ts DESC
                        LIMIT 14
                    ) AS tmp1
                    ORDER by start_ts
                ) AS tmp
            ), '{}'::json[]) AS history
        FROM repositories
        LEFT JOIN (
            SELECT
                *,
                row_number() over(PARTITION BY repository_id ORDER BY start_ts DESC) AS rn
            FROM runs
            WHERE status = 'failed'::run_status
        ) last_failed_subq ON last_failed_subq.repository_id = repositories.id AND last_failed_subq.rn = 1
        LEFT JOIN (
            SELECT
                *,
                row_number() over(PARTITION BY repository_id ORDER BY start_ts DESC) AS rn
            FROM runs
            WHERE type = 'fetch'::run_type AND status != 'interrupted'::run_status
        ) last_fetch_subq ON last_fetch_subq.repository_id = repositories.id AND last_fetch_subq.rn = 1
        LEFT JOIN (
            SELECT
                *,
                row_number() over(PARTITION BY repository_id ORDER BY start_ts DESC) AS rn
            FROM runs
            WHERE type = 'parse'::run_type AND status != 'interrupted'::run_status
        ) last_parse_subq ON last_parse_subq.repository_id = repositories.id AND last_parse_subq.rn = 1
        WHERE state != 'legacy'::repository_state
        ORDER BY sortname
    "#})
    .fetch_all(&state.pool)
    .await?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            repositories,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
