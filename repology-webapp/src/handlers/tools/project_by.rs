// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use anyhow::Result;
use askama::Template;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use indoc::indoc;
use itertools::Itertools;
use serde::Deserialize;

use crate::repository_data::{RepositoriesDataSnapshot, RepositoryData};
use crate::result::HandlerResult;
use crate::routes::{MyRoute, Route};
use crate::state::AppState;

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
    pub route: Route,
    pub target_type: TargetType,
}

impl TargetPage {
    pub const fn new(
        alias: &'static str,
        title: &'static str,
        route: Route,
        target_type: TargetType,
    ) -> Self {
        Self {
            alias,
            title,
            route,
            target_type,
        }
    }
}

const TARGET_PAGES: &[TargetPage] = &[
    TargetPage::new(
        "project_versions",
        "Project versions — /project/<name>/versions",
        Route::ProjectVersions,
        TargetType::Html,
    ),
    TargetPage::new(
        "project_packages",
        "Project packages — /project/<name>/packages",
        Route::ProjectPackages,
        TargetType::Html,
    ),
    TargetPage::new(
        "project_information",
        "Project information — /project/<name>/information",
        Route::ProjectInformation,
        TargetType::Html,
    ),
    TargetPage::new(
        "project_history",
        "Project history — /project/<name>/history",
        Route::ProjectHistory,
        TargetType::Html,
    ),
    TargetPage::new(
        "project_badges",
        "Project badges — /project/<name>/badges",
        Route::ProjectBadges,
        TargetType::Html,
    ),
    TargetPage::new(
        "project_reports",
        "Project reports — /project/<name>/reports",
        Route::ProjectReport,
        TargetType::Html,
    ),
    TargetPage::new(
        "badge_vertical_allrepos",
        "Vertical badge — /badge/vertical-allrepos/<name>.svg",
        Route::BadgeVerticalAllRepos,
        TargetType::Html,
    ),
    TargetPage::new(
        "badge_tiny_repos",
        "Tiny badge with number of repositories — /badge/tiny-repos/<name>.svg",
        Route::BadgeTinyRepos,
        TargetType::Html,
    ),
    TargetPage::new(
        "badge_latest_versions",
        "Tiny badge with latest packaged version(s) — /badge/tiny-versions/<name>.svg",
        Route::BadgeLatestVersions,
        TargetType::Html,
    ),
    TargetPage::new(
        "badge_version_for_repo",
        "Tiny badge with version for this repository — /badge/version-for-repo/<repo>/<name>.svg",
        Route::BadgeVersionForRepo,
        TargetType::Html,
    ),
    TargetPage::new(
        "api_v1_project",
        "API v1 project information — /api/v1/project/<name>",
        Route::ApiV1Project,
        TargetType::Json,
    ),
];

#[derive(Template)]
#[template(path = "tools/project-by.html")]
struct ConstructTemplateParams<'a> {
    my_route: &'a MyRoute,
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
    my_route: &'a MyRoute,
    query: &'a QueryParams,
    reason: FailureReason,
}

#[derive(Template)]
#[template(path = "tools/project-by/ambiguity.html")]
struct AmbiguityTemplateParams<'a> {
    my_route: &'a MyRoute,
    query: &'a QueryParams,
    targets: &'a [(String, String)],
    repository_data: &'a RepositoryData,
}

fn project_by_error(
    my_route: &MyRoute,
    query: QueryParams,
    reason: FailureReason,
) -> HandlerResult {
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
            my_route,
            query: &query,
            reason,
        }
        .render()?,
    )
        .into_response())
}

pub async fn project_by_perform(
    my_route: &MyRoute,
    query: QueryParams,
    gen_query: Vec<(String, String)>,
    state: &AppState,
    name: &str,
) -> HandlerResult {
    let Some(name_type) = query
        .name_type
        .as_ref()
        .filter(|name_type| *name_type == "binname" || *name_type == "srcname")
    else {
        // or else sql will fail; TODO: check this when parsing query
        return project_by_error(my_route, query, FailureReason::BadNameType);
    };

    let Some(repository_name) = &query.repo else {
        return project_by_error(my_route, query, FailureReason::RepositoryNotSpecified);
    };

    let repositories_data = state.repository_data_cache.snapshot();

    let Some(repository_data) = repositories_data.active_repository(repository_name) else {
        return project_by_error(my_route, query, FailureReason::RepositoryNotFound);
    };

    let Some(target_page) = TARGET_PAGES
        .iter()
        .find(|page| Some(page.alias) == query.target_page.as_deref())
    else {
        return project_by_error(my_route, query, FailureReason::BadTargetPage);
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

    let target_projects: Vec<_> = project_names
        .into_iter()
        .sorted()
        .map(|project_name| -> Result<(String, String)> {
            let mut path = target_page.route.url_for();

            path = path.path_param("project_name", &project_name)?;
            if target_page.route.path().contains("{repository_name}") {
                path = path.path_param("repository_name", &repository_data.name)?;
            }

            for (key, value) in gen_query.iter() {
                if !matches!(
                    key.as_ref(),
                    "repo" | "name_type" | "name" | "noautoresolve" | "target_page"
                ) {
                    path = path.query_param(key, value);
                }
            }

            Ok((project_name, path.build()?))
        })
        .try_collect()?;

    Ok(match (target_projects.len(), query.noautoresolve) {
        (0, _) => {
            return project_by_error(my_route, query, FailureReason::NotFound);
        }
        (1, _) | (_, false) => (
            StatusCode::FOUND,
            [(
                header::LOCATION,
                HeaderValue::from_maybe_shared(target_projects[0].1.clone())?,
            )],
        )
            .into_response(),
        _ => {
            let (content_type, body) = match target_page.target_type {
                TargetType::Html => (
                    mime::TEXT_HTML.as_ref(),
                    AmbiguityTemplateParams {
                        my_route,
                        query: &query,
                        targets: &target_projects,
                        repository_data,
                    }
                    .render()?,
                ),
                TargetType::Json => {
                    let targets = target_projects.into_iter().collect::<serde_json::Value>();
                    (
                        mime::APPLICATION_JSON.as_ref(),
                        serde_json::json!({
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

pub fn project_by_construct(
    my_route: &MyRoute,
    query: QueryParams,
    state: &AppState,
) -> HandlerResult {
    let target_page = TARGET_PAGES
        .iter()
        .find(|page| Some(page.alias) == query.target_page.as_deref());

    let template_url = match (&query.repo, &query.name_type, target_page) {
        (Some(_), Some(_), Some(_)) => Some(format!(
            "{}{}",
            crate::constants::REPOLOGY_HOSTNAME,
            // XXX: This URL has &name=<NAME> appended in the template. Since
            // template too has access to url_for_self, just pass a flag here
            my_route.url_for_self().build()?
        )),
        _ => None,
    };

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        ConstructTemplateParams {
            my_route,
            query: &query,
            template_url,
            repositories_data: &state.repository_data_cache.snapshot(),
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all, fields(query = ?query)))]
pub async fn project_by(
    my_route: MyRoute,
    Query(query): Query<QueryParams>,
    Query(gen_query): Query<Vec<(String, String)>>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult {
    if let Some(name) = &query.name.clone() {
        project_by_perform(&my_route, query, gen_query, &state, name).await
    } else {
        project_by_construct(&my_route, query, &state)
    }
}
