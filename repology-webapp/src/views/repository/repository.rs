// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use serde::Deserialize;
use sqlx::FromRow;

use repology_common::LinkType;

use crate::endpoints::Endpoint;
use crate::repository_data::SourceType;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub autorefresh: bool,
}

#[derive(Deserialize)]
pub struct RepositoryLink {
    url: String,
    #[serde(rename = "desc")]
    title: String,
}

#[derive(FromRow)]
pub struct Repository {
    pub title: String,
    pub source_type: SourceType,
    pub num_packages: i32,
    pub num_packages_newest: i32,
    pub num_packages_outdated: i32,
    pub num_packages_ignored: i32,
    pub num_projects: i32,
    pub num_projects_newest: i32,
    pub num_projects_outdated: i32,
    pub num_projects_unique: i32,
    pub num_problems: i32,
    pub num_maintainers: i32,
    pub repository_links: sqlx::types::Json<Vec<RepositoryLink>>,
    pub used_package_link_types: Vec<LinkType>,
    pub last_seen: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Template)]
#[template(path = "repository/gone.html")]
struct TemplateParamsGone {
    ctx: TemplateContext,
    repository: Repository,
}

#[derive(Template)]
#[template(path = "repository.html")]
struct TemplateParams<'a> {
    ctx: TemplateContext,
    repository_name: &'a str,
    repository: Repository,
    used_package_link_types: HashSet<LinkType>,
    autorefresh: bool,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn repository(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::Repository, gen_path, gen_query);

    let repository: Option<Repository> = sqlx::query_as(indoc! {r#"
        SELECT
            "desc" AS title,
            COALESCE(metadata->>'type', 'repository') AS source_type,
            num_packages,
            num_packages_newest,
            num_packages_outdated,
            num_packages_ignored,
            num_metapackages AS num_projects,
            num_metapackages_newest AS num_projects_newest,
            num_metapackages_outdated AS num_projects_outdated,
            num_metapackages_unique AS num_projects_unique,
            num_problems,
            num_maintainers,
            metadata,
            coalesce(metadata->'repolinks', '[]'::jsonb) AS repository_links,
            coalesce(used_package_link_types, '{}'::integer[]) AS used_package_link_types,
            last_seen,
            state!='legacy' AS is_active
        FROM repositories
        WHERE name = $1
    "#})
    .bind(&repository_name)
    .fetch_optional(&state.pool)
    .await?;

    let mut repository = match repository {
        Some(repository) if repository.is_active => repository,
        Some(repository) => {
            return Ok((
                StatusCode::NOT_FOUND, // or should it be GONE?
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
                )],
                TemplateParamsGone { ctx, repository }.render()?,
            )
                .into_response());
        }
        None => {
            return Ok((StatusCode::NOT_FOUND, "repository not found".to_owned()).into_response());
        }
    };

    let used_package_link_types: HashSet<_> =
        std::mem::take(&mut repository.used_package_link_types)
            .into_iter()
            .collect();

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            repository_name: &repository_name,
            repository,
            used_package_link_types,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
