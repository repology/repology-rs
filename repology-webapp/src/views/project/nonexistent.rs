// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::Template;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use indoc::indoc;
use sqlx::FromRow;
use tower_cookies::{Cookie, Cookies};

use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

use crate::views::projects::common::{
    CategorizedDisplayVersions, PackageForListing, ProjectForListing,
    packages_to_categorized_display_versions_per_project,
};

use super::common::Project;

#[derive(Template)]
#[template(path = "project/404.html")]
struct TemplateParams404 {
    ctx: TemplateContext,
    project_name: String,
    redirect_projects_list: Vec<ProjectListItem>,
}

#[derive(Template)]
#[template(path = "project/410.html")]
struct TemplateParams410 {
    ctx: TemplateContext,
    project_name: String,
    project: Project,
    redirect_projects_list: Vec<ProjectListItem>,
    leftovers_summary: ProjectLeftoversSummary,
}

#[derive(FromRow)]
pub struct ProjectLeftoversSummary {
    pub has_history: bool,
    pub has_reports: bool,
    pub has_cves: bool,
}

pub struct ProjectListItem {
    pub project: ProjectForListing,
    pub versions: CategorizedDisplayVersions,
}

#[cfg_attr(
    not(feature = "coverage"),
    tracing::instrument(skip_all, fields(project_name = project_name))
)]
pub async fn nonexisting_project(
    state: &AppState,
    cookies: &Cookies,
    ctx: TemplateContext,
    project_name: String,
    project: Option<Project>,
) -> EndpointResult {
    // note that we don't indicate to user that number of redirects per page is exceeded
    // as it's natually limited and cannot really exceed a few dozen
    let redirect_project_names: Vec<String> = sqlx::query_scalar(indoc! {"
            SELECT DISTINCT
                (SELECT effname FROM metapackages WHERE id = new.project_id)
            FROM project_redirects AS old
            INNER JOIN project_redirects AS new USING (repository_id, trackname)
            INNER JOIN metapackages ON(metapackages.id = new.project_id)
            WHERE
                old.project_id = (SELECT id FROM metapackages WHERE effname = $1)
                AND NOT old.is_actual
                AND new.is_actual
                AND metapackages.num_repos > 0
        UNION
            SELECT
                newname
            FROM project_redirects_manual
            WHERE
                oldname = $1
                AND EXISTS (
                    -- only return valid and active projects
                    SELECT *
                    FROM metapackages
                    WHERE metapackages.effname = project_redirects_manual.newname AND num_repos > 0
                )
        LIMIT $2
    "})
    .bind(&project_name)
    .bind(crate::constants::REDIRECTS_PER_PAGE as i32)
    .fetch_all(&state.pool)
    .await?;

    let (projects, packages) = match redirect_project_names.len() {
        1 => {
            // single redirect - follow it right away
            // TODO: we check redirect count here, however
            // we should instead check number of projects fetched
            // by redirect to exclude gone projects
            let target_project_name = &redirect_project_names[0][..];
            cookies.add(
                Cookie::build((format!("rdr_{target_project_name}"), project_name))
                    .path("/")
                    .max_age(tower_cookies::cookie::time::Duration::seconds(60))
                    .into(),
            );
            return Ok((
                StatusCode::MOVED_PERMANENTLY,
                [(
                    header::LOCATION,
                    HeaderValue::from_maybe_shared(
                        ctx.url_for(ctx.endpoint, &[("project_name", target_project_name)])?,
                    )?,
                )],
            )
                .into_response());
        }
        0 => Default::default(),
        _ => {
            // TODO: parallelize queries
            let projects: Vec<ProjectForListing> = sqlx::query_as(indoc! {"
                SELECT
                    effname,
                    num_families,
                    has_related
                FROM metapackages
                WHERE effname = ANY($1)
                ORDER BY effname
            "})
            .bind(&redirect_project_names)
            .fetch_all(&state.pool)
            .await?;

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
            .bind(&redirect_project_names)
            .fetch_all(&state.pool)
            .await?;

            (projects, packages)
        }
    };

    let mut versions_per_project =
        packages_to_categorized_display_versions_per_project(&packages, None, None);

    let redirect_projects_list = projects
        .into_iter()
        .map(|project| {
            let versions = versions_per_project
                .remove(&project.effname)
                .unwrap_or_default();
            ProjectListItem { project, versions }
        })
        .collect();

    if let Some(project) = project {
        let leftovers_summary: ProjectLeftoversSummary = sqlx::query_as(indoc! {"
            SELECT
                EXISTS (SELECT * FROM metapackages_events WHERE effname = $1) AS has_history,
                EXISTS (SELECT * FROM reports WHERE effname = $1) AS has_reports,
                EXISTS (SELECT * FROM vulnerable_projects WHERE effname = $1) AS has_cves;
        "})
        .bind(&project_name)
        .fetch_one(&state.pool)
        .await?;

        Ok((
            StatusCode::NOT_FOUND,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
            )],
            TemplateParams410 {
                ctx,
                project_name,
                project,
                redirect_projects_list,
                leftovers_summary,
            }
            .render()?,
        )
            .into_response())
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
            )],
            TemplateParams404 {
                ctx,
                project_name,
                redirect_projects_list,
            }
            .render()?,
        )
            .into_response())
    }
}
