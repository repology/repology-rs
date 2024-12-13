// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use indoc::indoc;

use crate::endpoints::Endpoint;
use crate::repository_data::RepositoryData;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

use super::common::Project;
use super::nonexistent::nonexisting_project;

#[derive(Template)]
#[template(path = "project/badges.html")]
struct TemplateParams<'a> {
    ctx: TemplateContext,
    project_name: &'a str,
    project: Option<Project>,
    containing_repositories_data: Vec<&'a RepositoryData>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn project_badges(
    Path(project_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new_without_params(Endpoint::ProjectBadges);

    let project: Option<Project> = sqlx::query_as(indoc! {"
        SELECT
            num_repos,
            has_cves,
            has_related,
            orphaned_at
        FROM metapackages
        WHERE effname = $1
    "})
    .bind(&project_name)
    .fetch_optional(&state.pool)
    .await?;

    if project
        .as_ref()
        .is_none_or(|project| project.num_repos == 0)
    {
        return nonexisting_project(&state, ctx, project_name, project).await;
    }

    let containing_repository_names: HashSet<String> = sqlx::query_scalar(indoc! {"
        SELECT DISTINCT repo FROM packages WHERE effname = $1
    "})
    .bind(&project_name)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .collect();

    let repositories_data = state.repository_data_cache.snapshot();

    let containing_repositories_data: Vec<_> = repositories_data
        .active_repositories()
        .into_iter()
        .filter(|repository| containing_repository_names.contains(&repository.name))
        .collect();

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            project_name: &project_name,
            project,
            containing_repositories_data,
        }
        .render()?,
    )
        .into_response())
}
