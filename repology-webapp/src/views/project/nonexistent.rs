// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::Template;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use indoc::indoc;
use sqlx::FromRow;

use crate::repository_data::RepositoryData;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

use crate::views::projects::common::{
    packages_to_categorized_display_versions_per_project, CategorizedDisplayVersions,
    PackageForListing, ProjectForListing,
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

#[tracing::instrument(skip(state, ctx, project))]
pub async fn nonexisting_project(
    state: AppState,
    ctx: TemplateContext,
    project_name: String,
    project: Option<Project>,
) -> EndpointResult {
    // note that we don't indicate to used that number of redirects per page is exceeded
    // as it's natually limited and cannot really exceed a few dozen
    let redirect_project_names: Vec<String> = sqlx::query_scalar(indoc! {"
            SELECT DISTINCT
                (SELECT effname FROM metapackages WHERE id = new.project_id)
            FROM project_redirects AS old INNER JOIN project_redirects AS new USING(repository_id, trackname)
            WHERE
                old.project_id = (SELECT id FROM metapackages WHERE effname = $1) AND
                NOT old.is_actual AND new.is_actual
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
    .bind(&(crate::constants::REDIRECTS_PER_PAGE as i32))
    .fetch_all(&state.pool)
    .await?;

    let (projects, packages) = match redirect_project_names.len() {
        1 => {
            // single redirect - follow it right away
            let project_name = &redirect_project_names[0][..];
            return Ok((
                StatusCode::MOVED_PERMANENTLY,
                [(
                    header::LOCATION,
                    HeaderValue::from_maybe_shared(
                        ctx.url_for(ctx.endpoint, &[("project_name", project_name)])?,
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
