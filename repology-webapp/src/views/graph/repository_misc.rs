// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;

use crate::graphs::{render_graph, GraphType};
use crate::result::EndpointResult;
use crate::state::AppState;

use super::common::GRAPH_PERIOD;

async fn graph_generic(
    state: &AppState,
    repository_name: &str,
    multiplier: f32,
    divident_field_name: &str,
    divisor_field_name: &str,
    stroke: &str,
) -> EndpointResult {
    if !state
        .repository_data_cache
        .snapshot()
        .is_repository_active(&repository_name)
    {
        return Ok((StatusCode::NOT_FOUND, "repository not found".to_owned()).into_response());
    }

    let points: Vec<(DateTime<Utc>, f32)> = sqlx::query_as(
        &format!(
            indoc! {r#"
                SELECT * FROM (
                    (
                        SELECT
                            ts AS timestamp,
                            CASE
                                WHEN {1} = 0 THEN NULL
                                ELSE {0}::real / {1}::real
                            END * $3 AS value
                        FROM repositories_history_new
                        WHERE repository_id = (SELECT id FROM repositories WHERE name = $1) AND ts < now() - $2
                        ORDER BY ts DESC
                        LIMIT 1
                    )
                    UNION ALL
                    (
                        SELECT
                            ts AS timestamp,
                            CASE
                                WHEN {1} = 0 THEN NULL
                                ELSE {0}::real / {1}::real
                            END * $3 AS value
                        FROM repositories_history_new
                        WHERE repository_id = (SELECT id FROM repositories WHERE name = $1) AND ts >= now() - $2
                        ORDER BY ts
                    )
                ) WHERE value IS NOT NULL
            "#},
            divident_field_name, divisor_field_name
        )
    )
        .bind(&repository_name)
        .bind(&GRAPH_PERIOD)
        .bind(&multiplier)
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
                .collect(),
            GraphType::Float,
            GRAPH_PERIOD,
            "",
            stroke,
        ),
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_problems_per_1000_projects(
    Path(repository_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(
        &*state,
        &repository_name,
        1000.0,
        "num_problems",
        "num_projects",
        "#c0c000",
    )
    .await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_per_maintainer(
    Path(repository_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(
        &*state,
        &repository_name,
        1.0,
        "num_projects",
        "num_maintainers",
        "#c0c000",
    )
    .await
}
