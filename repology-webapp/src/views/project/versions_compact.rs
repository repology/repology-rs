// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use indoc::indoc;
use itertools::Itertools;
use sqlx::FromRow;
use tower_cookies::{Cookie, Cookies};

use repology_common::{PackageFlags, PackageStatus};

use crate::endpoints::MyEndpoint;
use crate::package::summarization::DisplayVersion;
use crate::package::traits::{PackageWithFlags, PackageWithStatus, PackageWithVersion};
use crate::repository_data::RepositoriesDataSnapshot;
use crate::result::EndpointResult;
use crate::state::AppState;

use super::common::Project;
use super::nonexistent::nonexisting_project;

#[derive(FromRow)]
struct Package {
    repo: String,
    version: String,
    status: PackageStatus,
    flags: i32,
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

impl PackageWithStatus for Package {
    fn status(&self) -> PackageStatus {
        self.status
    }
}

#[derive(Template)]
#[template(path = "project/versions-compact.html")]
struct TemplateParams<'a> {
    endpoint: &'a MyEndpoint,
    project_name: String,
    project: Project,
    num_packages: usize,
    versions_by_repo: HashMap<String, Vec<DisplayVersion>>,
    repositories_data: &'a RepositoriesDataSnapshot,
    redirect_from: Option<String>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all, fields(project_name = project_name)))]
pub async fn project_versions_compact(
    endpoint: MyEndpoint,
    Path(project_name): Path<String>,
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
) -> EndpointResult {
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
        return nonexisting_project(&endpoint, &state, &cookies, project_name, None).await;
    };

    if project.is_orphaned() {
        return nonexisting_project(&endpoint, &state, &cookies, project_name, Some(project)).await;
    }

    // TODO: try fetching project and packages in parallel tasks, see
    // if this affects latency
    let packages: Vec<Package> = sqlx::query_as(indoc! {"
        SELECT
            repo,
            version,
            versionclass AS status,
            flags
        FROM packages
        WHERE effname = $1
    "})
    .bind(&project_name)
    .fetch_all(&state.pool)
    .await?;

    let num_packages = packages.len();
    let packages_by_repo = packages
        .into_iter()
        .into_group_map_by(|package| package.repo.clone());
    let versions_by_repo: HashMap<String, Vec<DisplayVersion>> = packages_by_repo
        .into_iter()
        .map(|(repo, packages)| {
            (
                repo,
                packages
                    .into_iter()
                    .map(|package| DisplayVersion::from_package(&package))
                    .sorted_by(|a, b| a.cmp(b).reverse())
                    .dedup()
                    .collect(),
            )
        })
        .collect();

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            endpoint: &endpoint,
            project_name,
            project,
            num_packages,
            versions_by_repo,
            repositories_data: &state.repository_data_cache.snapshot(),
            redirect_from,
        }
        .render()?,
    )
        .into_response())
}
