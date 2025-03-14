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
struct Project {
    project_name: String,
    delta: i32,
    last_change_timestamp: DateTime<Utc>,
    has_related: bool,
}

#[derive(Template)]
#[template(path = "tools/trending.html")]
struct TemplateParams {
    ctx: TemplateContext,
    trending_projects: Vec<Project>,
    declining_projects: Vec<Project>,
    autorefresh: bool,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn trending(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::Trending, gen_path, gen_query);

    // XXX: two queries take around 190ms, so this endpoint is a candidate
    // for in-state caching or executing both queries in parallel
    let trending_projects: Vec<Project> = sqlx::query_as(indoc! {r#"
        SELECT
            effname AS project_name,
            sum(delta)::integer AS delta,
            max(ts) FILTER (WHERE delta > 0) AS last_change_timestamp,
            (SELECT has_related FROM metapackages WHERE effname = project_turnover.effname) AS has_related
        FROM project_turnover
        WHERE ts >= now() - $1
        GROUP BY effname
        HAVING sum(delta) > 1
        ORDER BY delta DESC, last_change_timestamp DESC, effname
        LIMIT $2
    "#})
    .bind(crate::constants::TRENDING_PROJECTS_PERIOD)
    .bind(crate::constants::MAX_TRENDING_PROJECTS as i32)
    .fetch_all(&state.pool)
    .await?;

    let declining_projects: Vec<Project> = sqlx::query_as(indoc! {r#"
        SELECT
            effname AS project_name,
            sum(delta)::integer AS delta,
            max(ts) FILTER (WHERE delta < 0) AS last_change_timestamp,
            (SELECT has_related FROM metapackages WHERE effname = project_turnover.effname) AS has_related
        FROM project_turnover
        WHERE ts >= now() - $1
        GROUP BY effname
        HAVING sum(delta) < -1
        ORDER BY delta, last_change_timestamp DESC, effname
        LIMIT $2
    "#})
    .bind(crate::constants::DECLINING_PROJECTS_PERIOD)
    .bind(crate::constants::MAX_TRENDING_PROJECTS as i32)
    .fetch_all(&state.pool)
    .await?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            trending_projects,
            declining_projects,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
