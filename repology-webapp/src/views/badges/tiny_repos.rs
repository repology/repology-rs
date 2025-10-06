// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::badges::{Cell, DEFAULT_THEME, render_generic_badge};
use crate::result::EndpointResult;
use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(rename = "header")]
    pub caption: Option<String>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn badge_tiny_repos(
    Path(project_name): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<QueryParams>,
) -> EndpointResult {
    let num_families: Option<i16> =
        sqlx::query_scalar("SELECT num_families FROM metapackages WHERE effname = $1")
            .bind(project_name)
            .fetch_optional(&state.pool)
            .await?;

    let theme = &DEFAULT_THEME;
    let body = render_generic_badge(
        &[vec![
            Cell::new(
                query
                    .caption
                    .as_ref()
                    .map_or("in repositories", String::as_str),
            )
            .collapsible(true),
            Cell::new(&format!("{}", num_families.unwrap_or(0))).color(theme.color_nice),
        ]],
        None,
        0,
        &state.font_measurer,
        theme,
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
