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

    let points: Vec<(DateTime<Utc>, f32)> = sqlx::query_as(
        &format!(
            indoc! {r#"
                SELECT * FROM (
                    (
                        SELECT
                            ts AS timestamp,
                            CASE
                                WHEN num_projects_newest + num_projects_outdated = 0 THEN NULL
                                ELSE 100.0::real * {0}::real / (num_projects_newest + num_projects_outdated)::real
                            END AS value
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
                                WHEN num_projects_newest + num_projects_outdated = 0 THEN NULL
                                ELSE 100.0::real * {0}::real / (num_projects_newest + num_projects_outdated)::real
                            END AS value
                        FROM repositories_history_new
                        WHERE repository_id = (SELECT id FROM repositories WHERE name = $1) AND ts >= now() - $2
                        ORDER BY ts
                    )
                ) WHERE value IS NOT NULL
            "#},
            field_name
        )
    )
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
                .collect(),
            GraphType::Float,
            GRAPH_PERIOD,
            "%",
            stroke,
        ),
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_newest_percent(
    Path(repository_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(&state, &repository_name, "num_projects_newest", "#5cb85c").await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_outdated_percent(
    Path(repository_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(&state, &repository_name, "num_projects_outdated", "#d9534f").await
}
