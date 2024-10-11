// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{anyhow, Error};
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use serde::Deserialize;
use sqlx::FromRow;

use crate::endpoints::{Endpoint, Section};
use crate::repository_data::RepositoryData;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::static_files::STATIC_FILES;
use crate::url_for::UrlConstructor;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub autorefresh: bool,
}

#[derive(Template)]
#[template(path = "log.html")]
struct TemplateParams {
    endpoint: Endpoint,
    gen_path: Vec<(String, String)>,
    gen_query: Vec<(String, String)>,

    run: Run,
    repometadata: RepositoryData,
    lines: Vec<LogLine>,
    autorefresh: bool,
}

impl TemplateParams {
    pub fn url_for_static(&self, file_name: &str) -> Result<String, Error> {
        let name_map = STATIC_FILES.name_to_hashed_name_map();

        let hashed_file_name = name_map
            .get(file_name)
            .ok_or_else(|| anyhow!("unknown static file \"{}\"", file_name))?
            .to_string();

        Ok(UrlConstructor::new(Endpoint::StaticFile.path())
            .with_field("file_name", &hashed_file_name)
            .construct()?)
    }

    pub fn url_for<'a>(
        &self,
        endpoint: Endpoint,
        fields: &[(&'a str, &'a str)],
    ) -> Result<String, Error> {
        Ok(UrlConstructor::new(endpoint.path())
            .with_fields(fields.iter().cloned())
            .construct()?)
    }

    pub fn url_for_self<'a>(&self, fields: &[(&'a str, &'a str)]) -> Result<String, Error> {
        Ok(UrlConstructor::new(self.endpoint.path())
            .with_fields(self.gen_path.iter().map(|(k, v)| (k.as_ref(), v.as_ref())))
            .with_fields(self.gen_query.iter().map(|(k, v)| (k.as_ref(), v.as_ref())))
            .with_fields(fields.iter().cloned())
            .construct()?)
    }

    pub fn is_section(&self, section: Section) -> bool {
        self.endpoint.is_section(section)
    }

    pub fn needs_ipv6_notice(&self) -> bool {
        false
    }

    pub fn admin(&self) -> bool {
        false
    }

    pub fn experimental_enabled(&self) -> bool {
        false
    }

    pub fn is_repology_rs(&self) -> bool {
        true
    }

    // TODO: hack before askama 12.2 with built-in deref filter is released
    pub fn deref<T: Copy>(&self, r: &&T) -> T {
        **r
    }
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
            endpoint: Endpoint::Log,
            gen_path,
            gen_query,
            run,
            repometadata: repository_data,
            lines,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
