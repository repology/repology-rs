// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use indoc::indoc;
use serde::Serialize;
use sqlx::FromRow;

use repology_common::PackageStatus;

use crate::result::EndpointResult;
use crate::state::AppState;
use crate::views::projects::projects::QueryParams;
use crate::views::projects::query::{ProjectsFilter, query_listing_projects};

#[derive(Serialize, FromRow)]
pub struct ApiPackage {
    pub repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subrepo: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub srcname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binname: Option<String>,
    pub visiblename: String,

    pub version: String,
    //#[serde(skip_serializing_if = "Option::is_none")]  // Note: this is commented
    // for bug-to-bug compatibility with python webapp
    pub origversion: Option<String>,

    pub status: PackageStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub maintainers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub licenses: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulnerable: Option<bool>,
}

#[derive(FromRow)]
struct ApiPackageWithEffname {
    pub effname: String,
    #[sqlx(flatten)]
    pub package: ApiPackage,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn api_v1_projects(
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    api_v1_projects_generic(None, None, query, &state).await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn api_v1_projects_bounded(
    Path(bound): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    if let Some(end) = bound.strip_prefix("..") {
        api_v1_projects_generic(None, Some(end), query, &state).await
    } else {
        api_v1_projects_generic(Some(&bound), None, query, &state).await
    }
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
async fn api_v1_projects_generic(
    start_project_name: Option<&str>,
    end_project_name: Option<&str>,
    query: QueryParams,
    state: &AppState,
) -> EndpointResult {
    let filter = ProjectsFilter {
        start_project_name,
        end_project_name,
        limit: crate::constants::PROJECTS_PER_PAGE as i32,
        ..query.as_filter()
    };

    let projects = query_listing_projects(&state.pool, &filter).await?;

    let packages: Vec<ApiPackageWithEffname> = sqlx::query_as(indoc! {"
        SELECT
            effname,
            repo,
            subrepo,
            srcname,
            binname,
            visiblename,
            version,
            CASE WHEN rawversion = version THEN NULL ELSE rawversion END AS origversion,
            versionclass AS status,
            comment AS summary,
            coalesce(maintainers, '{}'::text[]) AS maintainers,
            coalesce(licenses, '{}'::text[]) AS licenses,
            CASE WHEN category IS NULL THEN '{}'::text[] ELSE ARRAY[category] END AS categories,
            NULLIF((flags & (1 << 16))::boolean, false) AS vulnerable
        FROM packages
        WHERE effname = ANY($1)
    "})
    .bind(
        projects
            .iter()
            .map(|project| project.effname.as_str())
            .collect::<Vec<_>>(),
    )
    .fetch_all(&state.pool)
    .await?;

    let mut project_packages: HashMap<String, Vec<ApiPackage>> = Default::default();

    packages.into_iter().for_each(|package| {
        project_packages
            .entry(package.effname)
            .or_default()
            .push(package.package)
    });

    let body = serde_json::to_string(&project_packages)?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        )],
        body,
    )
        .into_response())
}
