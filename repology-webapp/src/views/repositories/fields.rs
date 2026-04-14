// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use indoc::indoc;
use serde::Deserialize;
use sqlx::FromRow;

use crate::endpoints::MyEndpoint;
use crate::result::EndpointResult;
use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub autorefresh: bool,
}

#[derive(Template)]
#[template(path = "repositories/fields.html")]
struct TemplateParams<'a> {
    endpoint: &'a MyEndpoint,
    repositories: Vec<Repository>,
    autorefresh: bool,
}

// we have to use separate struct because we cannot fetch from DB into HashSets directly
#[derive(FromRow)]
struct DbRepository {
    name: String,
    title: String,
    source_type: String,

    fields: Vec<String>,
    link_types: Vec<i32>,
}

struct Repository {
    name: String,
    title: String,
    source_type: String,

    fields: HashSet<String>,
    link_types: HashSet<i32>,
}

impl From<DbRepository> for Repository {
    fn from(repository: DbRepository) -> Self {
        Self {
            name: repository.name,
            title: repository.title,
            source_type: repository.source_type,

            fields: repository.fields.into_iter().collect(),
            link_types: repository.link_types.into_iter().collect(),
        }
    }
}

#[cfg_attr(
    not(feature = "coverage"),
    tracing::instrument(skip_all, fields(query = ?query))
)]
pub async fn repositories_fields(
    endpoint: MyEndpoint,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let repositories: Vec<DbRepository> = sqlx::query_as(indoc! {r#"
        SELECT
            name,
            "desc" AS title,
            COALESCE(metadata->>'type', 'repository') AS source_type,
            COALESCE(used_package_fields, '{}'::text[]) AS fields,
            COALESCE(used_package_link_types, '{}'::integer[]) AS link_types
        FROM repositories
        WHERE state = 'active'
        ORDER BY sortname
    "#})
    .fetch_all(&state.pool)
    .await?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            endpoint: &endpoint,
            repositories: repositories.into_iter().map(|r| r.into()).collect(),
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
