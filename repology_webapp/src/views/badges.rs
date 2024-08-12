// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::result::EndpointResult;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::badges::{render_generic_badge, Cell};

#[derive(Deserialize)]
pub struct BadgeTinyReposQueryParams {
    pub header: Option<String>,
}

pub async fn badge_tiny_repos(
    Path(project_name): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<BadgeTinyReposQueryParams>,
) -> EndpointResult {
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
            //Cell::new(query.header.map_or("in repositories", |header| &header)),
            Cell::new(
                query
                    .header
                    .as_ref()
                    .map_or("in repositories", |header| header),
            ),
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
