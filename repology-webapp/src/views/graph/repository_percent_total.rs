// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use serde::Deserialize;

use crate::graphs::{render_graph, GraphType};
use crate::result::EndpointResult;
use crate::state::AppState;

use super::common::GRAPH_PERIOD;

fn get_true() -> bool {
    true
}

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default = "get_true")]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub experimental_history: bool,
}

async fn graph_generic(
    state: &AppState,
    repository_name: &str,
    experimental_history: bool,
    field_name: &str,
    stroke: &str,
) -> EndpointResult {
    if state
        .repository_data_cache
        .get_active(&repository_name)
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
                            CASE
                                WHEN num_projects = 0 THEN NULL
                                ELSE 100.0::real * {0}::real / num_projects::real
                            END AS value
                        FROM repositories_history_new
                        WHERE repository_id = (SELECT id FROM repositories WHERE name = $1) AND ts < now() - $3
                        ORDER BY ts DESC
                        LIMIT 1
                    )
                    UNION ALL
                    (
                        SELECT
                            ts AS timestamp,
                            CASE
                                WHEN num_projects = 0 THEN NULL
                                ELSE 100.0::real * {0}::real / num_projects::real
                            END AS value
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
                        CASE
                            WHEN (snapshot->$1->>'num_metapackages')::integer = 0 THEN NULL
                            ELSE 100.0::real * (snapshot->$1->>$2)::real / (snapshot->$1->>'num_metapackages')::real
                        END AS value
                    FROM repositories_history
                    WHERE ts < now() - $3
                    ORDER BY ts DESC
                    LIMIT 1
                )
                UNION ALL
                (
                    SELECT
                        ts AS timestamp,
                        CASE
                            WHEN (snapshot->$1->>'num_metapackages')::integer = 0 THEN NULL
                            ELSE 100.0::real * (snapshot->$1->>$2)::real / (snapshot->$1->>'num_metapackages')::real
                        END AS value
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
pub async fn graph_repository_projects_unique_percent(
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(
        &*state,
        &repository_name,
        query.experimental_history,
        "num_projects_unique",
        "#5bc0de",
    )
    .await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_problematic_percent(
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(
        &*state,
        &repository_name,
        query.experimental_history,
        "num_projects_problematic",
        "#808080",
    )
    .await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_repository_projects_vulnerable_percent(
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    graph_generic(
        &*state,
        &repository_name,
        query.experimental_history,
        "num_projects_vulnerable",
        "#ff0000",
    )
    .await
}
