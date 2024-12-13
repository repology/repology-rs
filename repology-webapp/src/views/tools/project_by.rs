// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use askama::Template;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use indoc::indoc;
use itertools::Itertools;
use serde::Deserialize;

use crate::endpoints::Endpoint;
use crate::repository_data::{RepositoriesDataSnapshot, RepositoryData};
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    pub repo: Option<String>,
    pub name_type: Option<String>,
    pub name: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub noautoresolve: bool,
    pub target_page: Option<String>,
}

enum TargetType {
    Html,
    Json,
}

struct TargetPage {
    pub alias: &'static str,
    pub title: &'static str,
    pub endpoint: Endpoint,
    pub target_type: TargetType,
}

impl TargetPage {
    pub const fn new(
        alias: &'static str,
        title: &'static str,
        endpoint: Endpoint,
        target_type: TargetType,
    ) -> Self {
        Self {
            alias,
            title,
            endpoint,
            target_type,
        }
    }
}

const TARGET_PAGES: &[TargetPage] = &[
    TargetPage::new(
        "project_versions",
        "Project versions — /project/<name>/versions",
        Endpoint::ProjectVersions,
        TargetType::Html,
    ),
    TargetPage::new(
        "project_packages",
        "Project packages — /project/<name>/packages",
        Endpoint::ProjectPackages,
        TargetType::Html,
    ),
    TargetPage::new(
        "project_information",
        "Project information — /project/<name>/information",
        Endpoint::ProjectInformation,
        TargetType::Html,
    ),
    TargetPage::new(
        "project_history",
        "Project history — /project/<name>/history",
        Endpoint::ProjectHistory,
        TargetType::Html,
    ),
    TargetPage::new(
        "project_badges",
        "Project badges — /project/<name>/badges",
        Endpoint::ProjectBadges,
        TargetType::Html,
    ),
    TargetPage::new(
        "project_reports",
        "Project reports — /project/<name>/reports",
        Endpoint::ProjectReport,
        TargetType::Html,
    ),
    TargetPage::new(
        "badge_vertical_allrepos",
        "Vertical badge — /badge/vertical-allrepos/<name>.svg",
        Endpoint::BadgeVerticalAllRepos,
        TargetType::Html,
    ),
    TargetPage::new(
        "badge_tiny_repos",
        "Tiny badge with number of repositories — /badge/tiny-repos/<name>.svg",
        Endpoint::BadgeTinyRepos,
        TargetType::Html,
    ),
    TargetPage::new(
        "badge_latest_versions",
        "Tiny badge with latest packaged version(s) — /badge/tiny-versions/<name>.svg",
        Endpoint::BadgeLatestVersions,
        TargetType::Html,
    ),
    TargetPage::new(
        "badge_version_for_repo",
        "Tiny badge with version for this repository — /badge/version-for-repo/<repo>/<name>.svg",
        Endpoint::BadgeVersionForRepo,
        TargetType::Html,
    ),
    TargetPage::new(
        "api_v1_project",
        "API v1 project information — /api/v1/project/<name>",
        Endpoint::ApiV1Project,
        TargetType::Json,
    ),
];

#[derive(Template)]
#[template(path = "tools/project-by.html")]
struct ConstructTemplateParams<'a> {
    ctx: TemplateContext,
    query: &'a QueryParams,
    template_url: Option<String>,
    repositories_data: &'a RepositoriesDataSnapshot,
}

#[derive(Clone, Copy, PartialEq)]
enum FailureReason {
    BadNameType,
    RepositoryNotSpecified,
    RepositoryNotFound,
    BadTargetPage,
    NotFound,
}

#[derive(Template)]
#[template(path = "tools/project-by/failed.html")]
struct FailureTemplateParams<'a> {
    ctx: &'a TemplateContext,
    query: &'a QueryParams,
    reason: FailureReason,
}

#[derive(Template)]
#[template(path = "tools/project-by/ambiguity.html")]
struct AmbiguityTemplateParams<'a> {
    ctx: &'a TemplateContext,
    query: &'a QueryParams,
    targets: &'a [(String, String)],
    repository_data: &'a RepositoryData,
}

fn project_by_error(
    ctx: TemplateContext,
    query: QueryParams,
    reason: FailureReason,
) -> EndpointResult {
    Ok((
        match reason {
            FailureReason::NotFound | FailureReason::RepositoryNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::BAD_REQUEST,
        },
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        FailureTemplateParams {
            ctx: &ctx,
            query: &query,
            reason,
        }
        .render()?,
    )
        .into_response())
}

pub async fn project_by_perform(
    query: QueryParams,
    gen_query: Vec<(String, String)>,
    state: &AppState,
    name: &str,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::ToolProjectBy, vec![], gen_query.clone());

    let name_type = if let Some(name_type) = query
        .name_type
        .as_ref()
        .filter(|name_type| *name_type == "binname" || *name_type == "srcname")
    {
        name_type
    } else {
        // or else sql will fail; TODO: check this when parsing query
        return project_by_error(ctx, query, FailureReason::BadNameType);
    };

    let repositories_data = state.repository_data_cache.snapshot();

    let repository_data = if let Some(repository_name) = &query.repo {
        if let Some(repository_data) = repositories_data.active_repository(repository_name) {
            repository_data
        } else {
            return project_by_error(ctx, query, FailureReason::RepositoryNotFound);
        }
    } else {
        return project_by_error(ctx, query, FailureReason::RepositoryNotSpecified);
    };

    let target_page = if let Some(target_page) = TARGET_PAGES
        .iter()
        .find(|page| Some(page.alias) == query.target_page.as_deref())
    {
        target_page
    } else {
        return project_by_error(ctx, query, FailureReason::BadTargetPage);
    };

    // Note: we don't need to check for project.num_packages > 0 because if a
    // project has no packages, it won't have names either
    let project_names: Vec<String> = sqlx::query_scalar(indoc! {"
        SELECT
            (SELECT effname FROM metapackages WHERE id = project_id)
        FROM
            project_names
        WHERE
            repository_id = $1 AND
            name_type = $2::project_name_type AND
            name = $3
    "})
    .bind(repository_data.id)
    .bind(name_type)
    .bind(name)
    .fetch_all(&state.pool)
    .await?;

    let params: Vec<_> = gen_query
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .filter(|(k, _)| {
            !matches!(
                *k,
                "repo" | "name_type" | "name" | "noautoresolve" | "target_page"
            )
        })
        .collect();

    let target_projects: Vec<_> = project_names
        .into_iter()
        .sorted()
        .map(|project_name| -> Result<(String, String)> {
            let mut params = params.clone();
            params.push(("project_name", project_name.as_str()));
            if target_page.endpoint.path().contains(":repository_name") {
                params.push(("repository_name", repository_data.name.as_str()));
            }

            let url = ctx.url_for(target_page.endpoint, &params)?;

            Ok((project_name, url))
        })
        .try_collect()?;

    Ok(match (target_projects.len(), query.noautoresolve) {
        (0, _) => {
            return project_by_error(ctx, query, FailureReason::NotFound);
        }
        (1, _) | (_, false) => (StatusCode::FOUND, [(
            header::LOCATION,
            HeaderValue::from_maybe_shared(target_projects[0].1.clone())?,
        )])
            .into_response(),
        _ => {
            use serde_json::json;
            let (content_type, body) = match target_page.target_type {
                TargetType::Html => (
                    mime::TEXT_HTML.as_ref(),
                    AmbiguityTemplateParams {
                        ctx: &ctx,
                        query: &query,
                        targets: &target_projects,
                        repository_data,
                    }
                    .render()?,
                ),
                TargetType::Json => {
                    let targets = target_projects.into_iter().collect::<HashMap<_, _>>();
                    (
                        mime::APPLICATION_JSON.as_ref(),
                        json!({
                            "_comment": "Ambiguous redirect, multiple target projects are possible",
                            "targets": targets,
                        })
                        .to_string(),
                    )
                }
            };

            (
                StatusCode::MULTIPLE_CHOICES,
                [(header::CONTENT_TYPE, HeaderValue::from_static(content_type))],
                body,
            )
                .into_response()
        }
    })
}

pub async fn project_by_construct(
    query: QueryParams,
    gen_query: Vec<(String, String)>,
    state: &AppState,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::ToolProjectBy, vec![], gen_query.clone());

    let target_page = TARGET_PAGES
        .iter()
        .find(|page| Some(page.alias) == query.target_page.as_deref());

    let template_url = match (&query.repo, &query.name_type, target_page) {
        (Some(_), Some(_), Some(_)) => Some(
            ctx.external_url_for_self(
                &gen_query
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect::<Vec<_>>(),
            )?,
        ),
        _ => None,
    };

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        ConstructTemplateParams {
            ctx,
            query: &query,
            template_url,
            repositories_data: &state.repository_data_cache.snapshot(),
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn project_by(
    Query(query): Query<QueryParams>,
    Query(gen_query): Query<Vec<(String, String)>>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    if let Some(name) = &query.name.clone() {
        project_by_perform(query, gen_query, &state, name).await
    } else {
        project_by_construct(query, gen_query, &state).await
    }
}
