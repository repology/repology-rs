// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

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
    multiplier: f32,
    divident_field_name: &str,
    divisor_field_name: &str,
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
                            CASE
                                WHEN {1} = 0 THEN NULL
                                ELSE {0}::real / {1}::real
                            END * $5 AS value
                        FROM repositories_history_new
                        WHERE repository_id = (SELECT id FROM repositories WHERE name = $1) AND ts < now() - $4
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
                            END * $5 AS value
                        FROM repositories_history_new
                        WHERE repository_id = (SELECT id FROM repositories WHERE name = $1) AND ts >= now() - $4
                        ORDER BY ts
                    )
                ) WHERE value IS NOT NULL
            "#},
            divident_field_name, divisor_field_name
        )
    } else {
        indoc! {r#"
            SELECT * FROM (
                (
                    SELECT
                        ts AS timestamp,
                        CASE
                            WHEN (snapshot->$1->>$3)::integer = 0 THEN NULL
                            ELSE (snapshot->$1->>$2)::real / (snapshot->$1->>$3)::real
                        END * $5 AS value
                    FROM repositories_history
                    WHERE ts < now() - $4
                    ORDER BY ts DESC
                    LIMIT 1
                )
                UNION ALL
                (
                    SELECT
                        ts AS timestamp,
                        CASE
                            WHEN (snapshot->$1->>$3)::integer = 0 THEN NULL
                            ELSE (snapshot->$1->>$2)::real / (snapshot->$1->>$3)::real
                        END * $5 AS value
                    FROM repositories_history
                    WHERE ts >= now() - $4
                    ORDER BY ts
                )
            ) WHERE value IS NOT NULL
        "#}
    };
    let points: Vec<(DateTime<Utc>, f32)> = sqlx::query_as(query)
        .bind(&repository_name)
        .bind(&divident_field_name.replace("num_projects", "num_metapackages"))
        .bind(&divisor_field_name.replace("num_projects", "num_metapackages"))
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
                .map(|(timestamp, value)| ((now - timestamp).to_std().unwrap(), value))
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
    Query(query): Query<QueryParams>,
    State(state): State<AppState>,
) -> EndpointResult {
    graph_generic(
        &state,
        &repository_name,
        query.experimental_history,
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
    Query(query): Query<QueryParams>,
    State(state): State<AppState>,
) -> EndpointResult {
    graph_generic(
        &state,
        &repository_name,
        query.experimental_history,
        1.0,
        "num_projects",
        "num_maintainers",
        "#c0c000",
    )
    .await
}
