// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use serde::Deserialize;
use sqlx::FromRow;

use crate::endpoints::Endpoint;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub autorefresh: bool,
}

#[derive(FromRow)]
struct Update {
    project_name: String,
    timestamp: DateTime<Utc>,
    spread: i32,
    versions: sqlx::types::Json<Vec<String>>,
}

#[derive(Template)]
#[template(path = "tools/important-updates.html")]
struct TemplateParams {
    ctx: TemplateContext,
    updates: Vec<Update>,
    autorefresh: bool,
}

#[cfg_attr(
    not(feature = "coverage"),
    tracing::instrument(skip(gen_path, gen_query, state))
)]
pub async fn important_updates(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::ImportantUpdates, gen_path, gen_query);

    let updates: Vec<Update> = sqlx::query_as(indoc! {r#"
        WITH ordered_recent_events AS (
            SELECT
                effname AS project_name,
                ts,
                spread::integer AS spread,
                data->'versions' AS versions,
                row_number() OVER(PARTITION BY effname ORDER BY ts DESC) AS rn
            FROM global_version_events
            WHERE
                type = 'newest_update'::global_version_event_type
                AND ts > now() - $1
                -- optimization: assume that after taking top N by spread
                -- in the following CTE, lowest spread will be higher than
                -- 5 (ATOW it was 7). This allows us to use partial index
                -- global_version_events_ts_idx_partial, don't forget to keep
                -- constant in its condition in sync
                AND spread > 5
        ), unicalized_recent_events_top AS (
            SELECT
                project_name,
                ts AS timestamp,
                spread,
                versions
            FROM ordered_recent_events
            -- XXX: this filters multiple events per project, but doesn't handle
            -- the case where version goes down (such as after new fake vesion was
            -- ignored), while we should exclude such events. Because of that, this
            -- query is not really production ready
            WHERE rn = 1
            ORDER BY spread DESC, project_name
            LIMIT $2
        )
        SELECT
            *
        FROM unicalized_recent_events_top
        ORDER BY spread DESC, timestamp DESC, project_name
    "#})
    .bind(crate::constants::IMPORTANT_UPDATES_AGE)
    .bind(crate::constants::IMPORTANT_UPDATES_MAX_COUNT as i32)
    .fetch_all(&state.pool)
    .await?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            updates,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
