// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use metrics::counter;
use serde::Deserialize;

use crate::badges::{render_generic_badge, Cell};
use crate::result::EndpointResult;
use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(rename = "header")]
    pub caption: Option<String>,
}

#[tracing::instrument(skip(state))]
pub async fn badge_tiny_repos(
    Path(project_name): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<QueryParams>,
) -> EndpointResult {
    counter!("repology_webapp.endpoints.requests_total", "endpoint" => "badge_tiny_repos")
        .increment(1);

    let project_name = if let Some(project_name) = project_name.strip_suffix(".svg") {
        project_name
    } else {
        return Ok((StatusCode::NOT_FOUND, "path must end with .svg".to_owned()).into_response());
    };

    let num_families: i64 =
        sqlx::query_scalar("SELECT count(DISTINCT family) FROM packages WHERE effname = $1;")
            .bind(project_name)
            .fetch_one(&state.pool)
            .await?;

    let body = render_generic_badge(
        &[vec![
            Cell::new(
                query
                    .caption
                    .as_ref()
                    .map_or("in repositories", String::as_str),
            )
            .collapsible(true),
            Cell::new(&format!("{}", num_families)).color("#007ec6"),
        ]],
        None,
        0,
        &state.font_measurer,
    )?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::IMAGE_SVG.as_ref()),
        )],
        body,
    )
        .into_response())
}
