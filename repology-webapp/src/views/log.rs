// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use serde::Deserialize;
use sqlx::FromRow;

use crate::endpoints::Endpoint;
use crate::repository_data::RepositoryData;
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
#[template(path = "log.html")]
struct TemplateParams {
    ctx: TemplateContext,

    run: Run,
    repometadata: RepositoryData,
    lines: Vec<LogLine>,
    autorefresh: bool,
}

#[derive(FromRow)]
struct Run {
    pub r#type: String,

    pub reponame: String,

    pub status: String,

    pub start_ts: DateTime<Utc>,
    pub finish_ts: Option<DateTime<Utc>>,

    #[sqlx(try_from = "i32")]
    pub num_lines: u64,
    #[sqlx(try_from = "i32")]
    pub num_warnings: u64,
    #[sqlx(try_from = "i32")]
    pub num_errors: u64,
}

#[derive(FromRow)]
struct LogLine {
    pub timestamp: DateTime<Utc>,
    pub severity: String,
    pub message: String,
}

#[tracing::instrument(skip(gen_path, gen_query, state))]
pub async fn log(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
    Path(run_id): Path<u64>,
    Query(query): Query<QueryParams>,
    State(state): State<AppState>,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::Log, gen_path, gen_query);

    // num_lines, num_warnings, num_errors is nullable and NULL for ongoing runs
    // to avoid using Option and extra if let in the template, we use coalesce
    // here to make these always defined; makes sense to refactor this
    let run: Option<Run> = sqlx::query_as(indoc! {r#"
        SELECT
            "type"::text,

            (SELECT name FROM repositories WHERE id = repository_id) AS reponame,

            status::text,

            start_ts,
            finish_ts,

            coalesce(num_lines, 0) AS num_lines,
            coalesce(num_warnings, 0) AS num_warnings,
            coalesce(num_errors, 0) AS num_errors
        FROM runs
        WHERE id = $1
    "#})
    .bind(run_id as i32)
    .fetch_optional(&state.pool)
    .await?;

    let run = if let Some(run) = run {
        run
    } else {
        return Ok((StatusCode::NOT_FOUND, "run not found".to_owned()).into_response());
    };

    let repository_data = state
        .repository_data_cache
        .get(&run.reponame)
        .await
        .expect("repository data should be available for run's repository");

    let lines: Vec<LogLine> = sqlx::query_as(indoc! {"
        SELECT
            timestamp,
            severity::text,
            message
        FROM log_lines
        WHERE run_id = $1
        ORDER BY lineno
    "})
    .bind(run_id as i32)
    .fetch_all(&state.pool)
    .await?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,

            run,
            repometadata: repository_data,
            lines,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
