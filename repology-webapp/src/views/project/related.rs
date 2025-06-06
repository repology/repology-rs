// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use indoc::indoc;
use sqlx::FromRow;
use tower_cookies::{Cookie, Cookies};

use crate::endpoints::Endpoint;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;
use crate::views::projects::common::CategorizedDisplayVersions;
use crate::views::projects::common::PackageForListing;
use crate::views::projects::common::packages_to_categorized_display_versions_per_project;

use super::common::Project;
use super::nonexistent::nonexisting_project;

#[derive(Debug, FromRow)]
struct RelatedProject {
    pub effname: String,
    pub rank: f64,
    #[sqlx(try_from = "i16")]
    pub num_families: u32,
    pub has_related: bool,
}

struct ProjectListItem {
    project: RelatedProject,
    versions: CategorizedDisplayVersions,
}

#[derive(Template)]
#[template(path = "project/related.html")]
struct TemplateParams {
    ctx: TemplateContext,
    project_name: String,
    project: Project,
    projects_list: Vec<ProjectListItem>,
    redirect_from: Option<String>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all, fields(project_name = project_name)))]
pub async fn project_related(
    Path(project_name): Path<String>,
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
) -> EndpointResult {
    let ctx = TemplateContext::new_without_params(Endpoint::ProjectRelated);

    let redirect_from_cookie_name = format!("rdr_{project_name}");
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

    let projects: Vec<RelatedProject> = sqlx::query_as(indoc! {"
        SELECT
            effname,
            rank,
            num_families,
            has_related
        FROM project_get_related(
            (SELECT id FROM metapackages WHERE effname=$1),
            $2
        )
        INNER JOIN metapackages ON (metapackages.id = related_project_id)
        ORDER BY rank DESC, effname;
    "})
    .bind(&project_name)
    .bind(crate::constants::PROJECTS_PER_PAGE as i32)
    .fetch_all(&state.pool)
    .await?;

    // XXX: we don't need to fetch repo and maintainers here as these are never used
    // in packages_to_categorized_display_versions_per_project(..., None, None). In
    // fact, we need to add specialization for focusless case.
    let packages: Vec<PackageForListing> = sqlx::query_as(indoc! {"
        SELECT
            '' AS repo,
            family,
            visiblename,
            effname,
            version,
            versionclass AS status,
            flags,
            '{}'::text[] AS maintainers
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

    let mut versions_per_project =
        packages_to_categorized_display_versions_per_project(&packages, None, None);

    let projects_list: Vec<_> = projects
        .into_iter()
        .map(|project| {
            let versions = versions_per_project
                .remove(&project.effname)
                .unwrap_or_default();
            ProjectListItem { project, versions }
        })
        .collect();

    Ok((
        if projects_list.is_empty() {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::OK
        },
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            project_name,
            project,
            projects_list,
            redirect_from,
        }
        .render()?,
    )
        .into_response())
}
