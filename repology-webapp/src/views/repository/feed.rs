// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use serde::Deserialize;
use sqlx::FromRow;

use crate::endpoints::Endpoint;
use crate::repository_data::RepositoryData;
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
struct Event {
    timestamp: DateTime<Utc>,
    project_name: String,
    event_type: String,
    data: serde_json::Value,
}

#[derive(Template)]
#[template(path = "repository/feed.html")]
struct TemplateParams<'a> {
    ctx: TemplateContext,
    events: Vec<Event>,
    repository_name: &'a str,
    repository_data: &'a RepositoryData,
    autorefresh: bool,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn repository_feed(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::RepositoryFeed, gen_path, gen_query);

    let repositories_data = state.repository_data_cache.snapshot();

    let Some(repository_data) = repositories_data.active_repository(&repository_name) else {
        return Ok((StatusCode::NOT_FOUND, "repository not found".to_owned()).into_response());
    };

    // XXX: this may be ineffective in some cases, as there may be a lot of events with a
    // single timestamp, and we only have index on (repository, ts), so postgres would have
    // to fetch thousands of events for the latest timestamp, then sort by id. However, this
    // is mostly OK in production as huge event clusters are rather rare and are quickly
    // displaced by other events.
    // I would also like to avoid adding unique column (e.g. id) to the index, as it grows
    // it from 50 to 200-250MB
    // However, it should not be a problem if we introduce better events table cleanup process,
    // removing events starting from 500th for each repo.
    // However, we may also want to introduce feed pagination...
    // See #58
    // Note: sorting by id (in addition to timestamp) in this query guarantees stable result
    let events: Vec<Event> = sqlx::query_as(indoc! {r#"
        WITH candidates AS (
            SELECT
                id,
                ts AS timestamp,
                metapackage_id AS project_id,
                type,
                data
            FROM repository_events
            WHERE
                repository_id = (SELECT id FROM repositories WHERE name = $1)
            ORDER BY timestamp DESC, id
            LIMIT $2
        )
        SELECT
            timestamp,
            type::text AS event_type,
            data,
            (SELECT effname FROM metapackages WHERE id = project_id) AS project_name
        FROM candidates
        ORDER BY timestamp DESC, project_name, type DESC, id
    "#})
    .bind(&repository_name)
    .bind(crate::constants::HTML_FEED_MAX_ENTRIES as i32)
    .fetch_all(&state.pool)
    .await?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            events,
            repository_name: &repository_name,
            repository_data,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
