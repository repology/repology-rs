// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;

use crate::graphs::{GraphType, render_graph};
use crate::result::EndpointResult;
use crate::state::AppState;

use super::common::GRAPH_PERIOD;

async fn graph_generic(
    state: &AppState,
    repository_name: &str,
    field_name: &str,
    stroke: &str,
) -> EndpointResult {
    if !state
        .repository_data_cache
        .snapshot()
        .is_repository_active(repository_name)
    {
        return Ok((StatusCode::NOT_FOUND, "repository not found".to_owned()).into_response());
    }

    let points: Vec<(DateTime<Utc>, f32)> = sqlx::query_as(&format!(
        indoc! {r#"
            SELECT * FROM (
                (
                    SELECT
                        ts AS timestamp,
                        {0}::real AS value
                    FROM repositories_history
                    WHERE repository_id = (SELECT id FROM repositories WHERE name = $1) AND ts < now() - $2
                    ORDER BY ts DESC
                    LIMIT 1
                )
                UNION ALL
                (
                    SELECT
                        ts AS timestamp,
                        {0}::real AS value
                    FROM repositories_history
                    WHERE repository_id = (SELECT id FROM repositories WHERE name = $1) AND ts >= now() - $2
                    ORDER BY ts
                )
            ) WHERE value IS NOT NULL
        "#},
        field_name
    ))
    .bind(repository_name)
    .bind(GRAPH_PERIOD)
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
                .map(|(timestamp, value)| ((now - timestamp).to_std().unwrap_or_default(), value))
                .collect::<Vec<_>>(),
            GraphType::Integer,
            GRAPH_PERIOD,
            "",
            stroke,
        ),
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_maintainers(
    Path(repository_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(&state, &repository_name, "num_maintainers", "#c000c0").await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_problems(
    Path(repository_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(&state, &repository_name, "num_problems", "#c00000").await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_total(
    Path(repository_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(&state, &repository_name, "num_projects", "#000").await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_unique(
    Path(repository_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(&state, &repository_name, "num_projects_unique", "#5bc0de").await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_newest(
    Path(repository_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(&state, &repository_name, "num_projects_newest", "#5cb85c").await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_outdated(
    Path(repository_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(&state, &repository_name, "num_projects_outdated", "#d9534f").await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_problematic(
    Path(repository_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(
        &state,
        &repository_name,
        "num_projects_problematic",
        "#808080",
    )
    .await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_vulnerable(
    Path(repository_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(
        &state,
        &repository_name,
        "num_projects_vulnerable",
        "#ff0000",
    )
    .await
}
