// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use indoc::indoc;
use tower_cookies::{Cookie, Cookies};

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
    project: Project,
    containing_repositories_data: Vec<&'a RepositoryData>,
    redirect_from: Option<String>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all, fields(project_name = project_name)))]
pub async fn project_badges(
    Path(project_name): Path<String>,
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
) -> EndpointResult {
    let ctx = TemplateContext::new_without_params(Endpoint::ProjectBadges);

    let redirect_from_cookie_name = format!("rdr_{}", project_name);
    let redirect_from = if let Some(cookie) = cookies.get(&redirect_from_cookie_name) {
        let value = cookie.value().to_string();
        cookies.remove(Cookie::build(redirect_from_cookie_name).path("/").into());
        Some(value)
    } else {
        None
    };

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

    let Some(project) = project else {
        return nonexisting_project(&state, &cookies, ctx, project_name, None).await;
    };

    if project.is_orphaned() {
        return nonexisting_project(&state, &cookies, ctx, project_name, Some(project)).await;
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
            redirect_from,
        }
        .render()?,
    )
        .into_response())
}
