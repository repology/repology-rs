// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;
use indoc::indoc;
use itertools::Itertools;
use sqlx::FromRow;

use repology_common::{PackageFlags, PackageStatus};

use crate::endpoints::Endpoint;
use crate::package::ordering::by_version_descending;
use crate::package::traits::{PackageWithFlags, PackageWithVersion};
use crate::repository_data::RepositoryData;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

use super::common::Project;
use super::nonexistent::nonexisting_project;

#[derive(FromRow)]
pub struct Package {
    pub repo: String,
    pub subrepo: Option<String>,
    pub visiblename: String,
    pub origversion: String,
    pub maintainers: Vec<String>,
    pub category: Option<String>,
    pub url: Option<String>,
    pub version: String,
    pub status: PackageStatus,
    pub flags: i32,
}

impl PackageWithVersion for Package {
    fn version(&self) -> &str {
        &self.version
    }
}
impl PackageWithFlags for Package {
    fn flags(&self) -> PackageFlags {
        PackageFlags::from_bits(self.flags as u32).expect("flags must be deserializable")
    }
}

#[derive(Template)]
#[template(path = "project/versions.html")]
struct TemplateParams {
    ctx: TemplateContext,
    project_name: String,
    project: Option<Project>,
    num_packages: usize,
    packages_by_repo: HashMap<String, Vec<Package>>,
    repositories_data: Vec<RepositoryData>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn project_versions(
    Path(project_name): Path<String>,
    State(state): State<AppState>,
) -> EndpointResult {
    let ctx = TemplateContext::new_without_params(Endpoint::ProjectVersions);

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
        return nonexisting_project(state, ctx, project_name, project).await;
    }

    // TODO: try fetching project and packages in parallel tasks, see
    // if this affects latency
    let packages: Vec<Package> = sqlx::query_as(indoc! {"
        SELECT
            repo,
            subrepo,
            visiblename,
            origversion,
            coalesce(maintainers, '{}'::text[]) AS maintainers,
            category,
            version,
            versionclass AS status,
            flags,
            (
                SELECT url
                FROM links
                WHERE id = (
                    WITH expanded_links AS (
                        SELECT
                            (tuple->>0)::integer AS link_type,
                            (tuple->>1)::integer AS link_id,
                            ordinality
                        FROM json_array_elements(links) WITH ORDINALITY AS t(tuple, ordinality)
                    )
                    SELECT
                        link_id
                    FROM expanded_links
                    WHERE
                        link_type IN (
                            4,  -- PROJECT_HOMEPAGE
                            5,  -- PACKAGE_HOMEPAGE
                            7,  -- PACKAGE_REPOSITORY
                            9,  -- PACKAGE_RECIPE
                            10  -- PACKAGE_RECIPE_RAW
                        )
                    ORDER BY ordinality -- or link_type, ordinality
                    LIMIT 1
                ) --AND coalesce(ipv4_success, true)  -- XXX: better display link status
            ) AS url
        FROM packages
        WHERE effname = $1
    "})
    .bind(&project_name)
    .fetch_all(&state.pool)
    .await?;

    let num_packages = packages.len();
    let mut packages_by_repo = packages
        .into_iter()
        .into_group_map_by(|package| package.repo.clone());
    packages_by_repo
        .values_mut()
        .for_each(|packages| by_version_descending::sort(packages));

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            project_name,
            project,
            num_packages,
            packages_by_repo,
            repositories_data: state.repository_data_cache.get_all_active().await,
        }
        .render()?,
    )
        .into_response())
}
