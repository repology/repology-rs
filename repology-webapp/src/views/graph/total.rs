// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::extract::State;
use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use sqlx::PgPool;

use crate::graphs::{render_graph, GraphType};
use crate::result::EndpointResult;
use crate::state::AppState;

use super::common::GRAPH_PERIOD;

async fn graph_generic(pool: &PgPool, field_name: &str, stroke: &str) -> EndpointResult {
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
            "",
            stroke,
        ),
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_total_packages(State(state): State<AppState>) -> EndpointResult {
    graph_generic(&state.pool, "num_packages", "#000").await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_total_projects(State(state): State<AppState>) -> EndpointResult {
    graph_generic(&state.pool, "num_metapackages", "#000").await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_total_maintainers(State(state): State<AppState>) -> EndpointResult {
    graph_generic(&state.pool, "num_maintainers", "#c000c0").await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_total_problems(State(state): State<AppState>) -> EndpointResult {
    graph_generic(&state.pool, "num_problems", "#c00000").await
}
