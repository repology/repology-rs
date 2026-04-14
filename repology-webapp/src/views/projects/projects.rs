// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use indoc::indoc;
use serde::Deserialize;

use crate::endpoints::MyEndpoint;
use crate::repository_data::RepositoriesDataSnapshot;
use crate::result::EndpointResult;
use crate::state::AppState;

use super::common::{
    CategorizedDisplayVersions, PackageForListing, ProjectForListing,
    packages_to_categorized_display_versions_per_project,
};
use super::query::{ProjectsFilter, query_listing_projects};

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    pub search: String,
    #[serde(default)]
    pub maintainer: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub inrepo: String,
    #[serde(default)]
    pub notinrepo: String,
    #[serde(default)]
    pub repos: String,
    #[serde(default)]
    pub families: String,
    #[serde(default)]
    pub repos_newest: String,
    #[serde(default)]
    pub families_newest: String,
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub newest: bool,
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub outdated: bool,
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub problematic: bool,
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub vulnerable: bool,
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub has_related: bool,
}

impl QueryParams {
    fn parse_range(range: &str) -> (Option<i32>, Option<i32>) {
        if let Some((start, end)) = range.split_once("-") {
            (start.parse::<i32>().ok(), end.parse::<i32>().ok())
        } else {
            let single = range.parse::<i32>().ok();
            (single, single)
        }
    }

    pub fn as_filter(&self) -> ProjectsFilter<'_> {
        let repositories_range = Self::parse_range(&self.repos);
        let families_range = Self::parse_range(&self.families);
        let repositories_newest_range = Self::parse_range(&self.repos_newest);
        let families_newest_range = Self::parse_range(&self.families_newest);

        ProjectsFilter {
            project_name_substring: Some(self.search.as_str()).filter(|s| !s.is_empty()),
            maintainer: Some(self.maintainer.as_str()).filter(|s| !s.is_empty()),
            in_repo: Some(self.inrepo.as_str()).filter(|s| !s.is_empty()),
            not_in_repo: Some(self.notinrepo.as_str()).filter(|s| !s.is_empty()),
            min_repositories: repositories_range.0,
            max_repositories: repositories_range.1,
            min_families: families_range.0,
            max_families: families_range.1,
            min_repositories_newest: repositories_newest_range.0,
            max_repositories_newest: repositories_newest_range.1,
            min_families_newest: families_newest_range.0,
            max_families_newest: families_newest_range.1,
            category: Some(self.category.as_str()).filter(|s| !s.is_empty()),
            require_newest: self.newest,
            require_outdated: self.outdated,
            require_problematic: self.problematic,
            require_has_related: self.has_related,
            require_vulnerable: self.vulnerable,
            ..Default::default()
        }
    }

    fn is_advanced(&self) -> bool {
        [
            &self.maintainer,
            &self.category,
            &self.inrepo,
            &self.notinrepo,
            &self.repos,
            &self.families,
            &self.repos_newest,
            &self.families_newest,
        ]
        .into_iter()
        .any(|s| !s.is_empty())
            || self.newest
            || self.outdated
            || self.problematic
            || self.vulnerable
            || self.has_related
    }

    fn add_params(&self, mut builder: axum_myroutes::PathBuilder) -> axum_myroutes::PathBuilder {
        if !self.search.is_empty() {
            builder = builder.query_param("search", &self.search);
        }
        if !self.maintainer.is_empty() {
            builder = builder.query_param("maintainer", &self.maintainer)
        }
        if !self.category.is_empty() {
            builder = builder.query_param("category", &self.category)
        }
        if !self.inrepo.is_empty() {
            builder = builder.query_param("inrepo", &self.inrepo)
        }
        if !self.notinrepo.is_empty() {
            builder = builder.query_param("notinrepo", &self.notinrepo)
        }
        if !self.repos.is_empty() {
            builder = builder.query_param("repos", &self.repos)
        }
        if !self.families.is_empty() {
            builder = builder.query_param("families", &self.families)
        }
        if !self.repos_newest.is_empty() {
            builder = builder.query_param("repos_newest", &self.repos_newest)
        }
        if !self.families_newest.is_empty() {
            builder = builder.query_param("families_newest", &self.families_newest);
        }
        if self.newest {
            builder = builder.query_param("newest", 1);
        }
        if self.outdated {
            builder = builder.query_param("outdated", 1);
        }
        if self.problematic {
            builder = builder.query_param("problematic", 1);
        }
        if self.vulnerable {
            builder = builder.query_param("vulnerable", 1);
        }
        if self.has_related {
            builder = builder.query_param("has_related", 1);
        }
        builder
    }
}

struct ProjectListItem {
    project: ProjectForListing,
    versions: CategorizedDisplayVersions,
}

#[derive(Template)]
#[template(path = "projects/index.html")]
struct TemplateParams<'a> {
    endpoint: &'a MyEndpoint,
    query: QueryParams,
    repositories_data: &'a RepositoriesDataSnapshot,
    projects_list: Vec<ProjectListItem>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all, fields(query = ?query)))]
pub async fn projects(
    endpoint: MyEndpoint,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    projects_generic(&endpoint, None, None, query, &state).await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all, fields(bound = bound, query = ?query)))]
pub async fn projects_bounded(
    endpoint: MyEndpoint,
    Path(bound): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    if let Some(end) = bound.strip_prefix("..") {
        projects_generic(&endpoint, None, Some(end), query, &state).await
    } else {
        projects_generic(&endpoint, Some(&bound), None, query, &state).await
    }
}

async fn projects_generic(
    endpoint: &MyEndpoint,
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

    let packages: Vec<PackageForListing> = sqlx::query_as(indoc! {"
        SELECT
            repo,
            family,
            visiblename,
            effname,
            version,
            versionclass AS status,
            flags,
            coalesce(maintainers, '{}'::text[]) AS maintainers
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

    let mut versions_per_project = packages_to_categorized_display_versions_per_project(
        &packages,
        Some(query.inrepo.as_str()).filter(|s| !s.is_empty()),
        Some(query.maintainer.as_str()).filter(|s| !s.is_empty()),
    );

    let projects_list = projects
        .into_iter()
        .map(|project| {
            let versions = versions_per_project
                .remove(&project.effname)
                .unwrap_or_default();
            ProjectListItem { project, versions }
        })
        .collect();

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            endpoint,
            query,
            repositories_data: &state.repository_data_cache.snapshot(),
            projects_list,
        }
        .render()?,
    )
        .into_response())
}
