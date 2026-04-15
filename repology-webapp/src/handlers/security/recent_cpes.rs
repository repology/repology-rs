// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use serde::Deserialize;
use sqlx::FromRow;

use crate::result::HandlerResult;
use crate::routes::MyRoute;
use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub autorefresh: bool,
}

#[derive(Template)]
#[template(path = "security/recent-cpes.html")]
struct TemplateParams<'a> {
    my_route: &'a MyRoute,
    cpes: Vec<Cpe>,
    autorefresh: bool,
}

#[derive(FromRow)]
struct Cpe {
    pub project_name: String,
    pub cpe_vendor: String,
    pub cpe_product: String,
    pub cpe_edition: String,
    pub cpe_lang: String,
    pub cpe_sw_edition: String,
    pub cpe_target_sw: String,
    pub cpe_target_hw: String,
    pub cpe_other: String,
    pub added_ts: DateTime<Utc>,
}

#[cfg_attr(
    not(coverage),
    tracing::instrument(skip_all, fields(query = ?query))
)]
pub async fn recent_cpes(
    my_route: MyRoute,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult {
    let cpes: Vec<Cpe> = sqlx::query_as(indoc! {r#"
        SELECT
            effname AS project_name,
            cpe_vendor,
            cpe_product,
            cpe_edition,
            cpe_lang,
            cpe_sw_edition,
            cpe_target_sw,
            cpe_target_hw,
            cpe_other,
            added_ts
        FROM manual_cpes
        ORDER BY added_ts DESC
        LIMIT $1
    "#})
    .bind(crate::constants::RECENT_CPES_MAX_COUNT as i32)
    .fetch_all(&state.pool)
    .await?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            my_route: &my_route,
            cpes,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
