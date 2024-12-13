// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use sqlx::FromRow;

use crate::endpoints::Endpoint;
use crate::feeds::{EventWithTimestamp, unicalize_feed_timestamps};
use crate::repository_data::RepositoryData;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

#[derive(FromRow)]
struct Event {
    id: i32,
    timestamp: DateTime<Utc>,
    project_name: String,
    event_type: String,
    data: serde_json::Value,
}

impl EventWithTimestamp for Event {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn set_timestamp(&mut self, timestamp: DateTime<Utc>) {
        self.timestamp = timestamp;
    }
}

#[derive(Template)]
#[template(path = "atom-feeds/repository/feed.xml")]
struct TemplateParams<'a> {
    ctx: TemplateContext,
    events: Vec<Event>,
    repository_name: &'a str,
    repository_data: &'a RepositoryData,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn repository_feed_atom(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
    Path(repository_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::RepositoryFeedAtom, gen_path, gen_query);

    let repositories_data = state.repository_data_cache.snapshot();

    let repository_data =
        if let Some(repository_data) = repositories_data.active_repository(&repository_name) {
            repository_data
        } else {
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
    let mut events: Vec<Event> = sqlx::query_as(indoc! {r#"
        WITH candidates_unlimited AS (
            SELECT
                id,
                ts AS timestamp,
                metapackage_id AS project_id,
                type,
                data,
                row_number() OVER (ORDER BY ts DESC, id) AS row_number
            FROM repository_events
            WHERE
                repository_id = (SELECT id FROM repositories WHERE name = $1)
            ORDER BY timestamp DESC, id
            LIMIT $2
        ), candidates AS (
            SELECT * FROM candidates_unlimited
            WHERE timestamp > now() - $3 OR row_number < $4
        )
        SELECT
            id,
            timestamp,
            type::text AS event_type,
            data,
            (SELECT effname FROM metapackages WHERE id = project_id) AS project_name
        FROM candidates
        ORDER BY timestamp DESC, project_name, type DESC, id
    "#})
    .bind(&repository_name)
    .bind(crate::constants::ATOM_FEED_MAX_ENTRIES as i32)
    .bind(crate::constants::ATOM_FEED_MAX_AGE)
    .bind(crate::constants::ATOM_FEED_MIN_ENTRIES as i32)
    .fetch_all(&state.pool)
    .await?;

    unicalize_feed_timestamps(&mut events);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/atom+xml"),
        )],
        TemplateParams {
            ctx,
            events,
            repository_name: &repository_name,
            repository_data,
        }
        .render()?,
    )
        .into_response())
}
