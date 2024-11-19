// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use serde::Deserialize;
use sqlx::PgPool;

use crate::graphs::{render_graph, GraphType};
use crate::result::EndpointResult;
use crate::state::AppState;

const GRAPH_PERIOD: Duration = Duration::from_days(21);

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub experimental_history: bool,
}

async fn graph_repository_generic(
    state: &AppState,
    repository_name: &str,
    experimental_history: bool,
    field_name: &str,
    stroke: &str,
) -> EndpointResult {
    if state
        .repository_data_cache
        .get_active(&repository_name)
        .await
        .is_none()
    {
        return Ok((StatusCode::NOT_FOUND, "repository not found".to_owned()).into_response());
    };

    let query = if experimental_history {
        &format!(
            indoc! {r#"
            SELECT * FROM (
                (
                    SELECT
                        ts AS timestamp,
                        {0}::real AS value
                    FROM repositories_history_new
                    WHERE repository_id = (SELECT id FROM repositories WHERE name = $1) AND ts < now() - $3
                    ORDER BY ts DESC
                    LIMIT 1
                )
                UNION ALL
                (
                    SELECT
                        ts AS timestamp,
                        {0}::real AS value
                    FROM repositories_history_new
                    WHERE repository_id = (SELECT id FROM repositories WHERE name = $1) AND ts >= now() - $3
                    ORDER BY ts
                )
            ) WHERE value IS NOT NULL
        "#},
            field_name
        )
    } else {
        indoc! {r#"
            SELECT * FROM (
                (
                    SELECT
                        ts AS timestamp,
                        (snapshot->$1->>$2)::real AS value
                    FROM repositories_history
                    WHERE ts < now() - $3
                    ORDER BY ts DESC
                    LIMIT 1
                )
                UNION ALL
                (
                    SELECT
                        ts AS timestamp,
                        (snapshot->$1->>$2)::real AS value
                    FROM repositories_history
                    WHERE ts >= now() - $3
                    ORDER BY ts
                )
            ) WHERE value IS NOT NULL
        "#}
    };
    let points: Vec<(DateTime<Utc>, f32)> = sqlx::query_as(query)
        .bind(&repository_name)
        .bind(&field_name.replace("num_projects", "num_metapackages"))
        .bind(&GRAPH_PERIOD)
        .fetch_all(&state.pool)
        .await?;

    let now = Utc::now();

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::IMAGE_SVG.as_ref()),
        )],
        render_graph(
            &points
                .into_iter()
                .map(|(timestamp, value)| ((now - timestamp).to_std().unwrap(), value))
                .collect(),
            GraphType::Integer,
            GRAPH_PERIOD,
            stroke,
        ),
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_maintainers(
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<AppState>,
) -> EndpointResult {
    graph_repository_generic(
        &state,
        &repository_name,
        query.experimental_history,
        "num_maintainers",
        "#c000c0",
    )
    .await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_problems(
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<AppState>,
) -> EndpointResult {
    graph_repository_generic(
        &state,
        &repository_name,
        query.experimental_history,
        "num_problems",
        "#c00000",
    )
    .await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_total(
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<AppState>,
) -> EndpointResult {
    graph_repository_generic(
        &state,
        &repository_name,
        query.experimental_history,
        "num_projects",
        "#000",
    )
    .await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_unique(
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<AppState>,
) -> EndpointResult {
    graph_repository_generic(
        &state,
        &repository_name,
        query.experimental_history,
        "num_projects_unique",
        "#5bc0de",
    )
    .await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_newest(
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<AppState>,
) -> EndpointResult {
    graph_repository_generic(
        &state,
        &repository_name,
        query.experimental_history,
        "num_projects_newest",
        "#5cb85c",
    )
    .await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_outdated(
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<AppState>,
) -> EndpointResult {
    graph_repository_generic(
        &state,
        &repository_name,
        query.experimental_history,
        "num_projects_outdated",
        "#d9534f",
    )
    .await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_problematic(
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<AppState>,
) -> EndpointResult {
    graph_repository_generic(
        &state,
        &repository_name,
        query.experimental_history,
        "num_projects_problematic",
        "#808080",
    )
    .await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_vulnerable(
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<AppState>,
) -> EndpointResult {
    graph_repository_generic(
        &state,
        &repository_name,
        query.experimental_history,
        "num_projects_vulnerable",
        "#ff0000",
    )
    .await
}

async fn graph_total_generic(pool: &PgPool, field_name: &str, stroke: &str) -> EndpointResult {
    let points: Vec<(DateTime<Utc>, f32)> = sqlx::query_as(indoc! {r#"
        SELECT * FROM (
            (
                SELECT
                    ts AS timestamp,
                    (snapshot->>$1)::real AS value
                FROM statistics_history
                WHERE ts < now() - $2
                ORDER BY ts DESC
                LIMIT 1
            )
            UNION ALL
            (
                SELECT
                    ts AS timestamp,
                    (snapshot->>$1)::real AS value
                FROM statistics_history
                WHERE ts >= now() - $2
                ORDER BY ts
            )
        ) WHERE value IS NOT NULL
    "#})
    .bind(&field_name)
    .bind(&GRAPH_PERIOD)
    .fetch_all(pool)
    .await?;

    let now = Utc::now();

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::IMAGE_SVG.as_ref()),
        )],
        render_graph(
            &points
                .into_iter()
                .map(|(timestamp, value)| ((now - timestamp).to_std().unwrap(), value))
                .collect(),
            GraphType::Integer,
            GRAPH_PERIOD,
            stroke,
        ),
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_total_packages(State(state): State<AppState>) -> EndpointResult {
    graph_total_generic(&state.pool, "num_packages", "#000").await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_total_projects(State(state): State<AppState>) -> EndpointResult {
    graph_total_generic(&state.pool, "num_metapackages", "#000").await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_total_maintainers(State(state): State<AppState>) -> EndpointResult {
    graph_total_generic(&state.pool, "num_maintainers", "#c000c0").await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_total_problems(State(state): State<AppState>) -> EndpointResult {
    graph_total_generic(&state.pool, "num_problems", "#c00000").await
}
