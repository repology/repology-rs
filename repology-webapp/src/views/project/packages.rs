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

use repology_common::{LinkType, PackageFlags, PackageStatus};

use crate::endpoints::Endpoint;
use crate::package::ordering::by_name_asc_version_desc;
use crate::package::traits::{PackageWithDisplayName, PackageWithFlags, PackageWithVersion};
use crate::repository_data::RepositoriesDataSnapshot;
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
    pub rawversion: String,
    pub maintainers: Vec<String>,
    pub category: Option<String>,
    pub summary: Option<String>,
    pub licenses: Vec<String>,
    pub version: String,
    pub status: PackageStatus,
    pub flags: i32,
    pub links: sqlx::types::Json<Vec<(LinkType, i32, Option<String>)>>,
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
impl PackageWithDisplayName for Package {
    fn display_name(&self) -> &str {
        &self.visiblename
    }
}

#[derive(FromRow)]
struct Link {
    id: i32,
    url: String,
    ipv4_success: Option<bool>,
    has_ipv4_permanent_redirect: bool,
    ipv6_success: Option<bool>,
    has_ipv6_permanent_redirect: bool,
}

#[derive(Template)]
#[template(path = "project/packages.html")]
struct TemplateParams<'a> {
    ctx: TemplateContext,
    project_name: &'a str,
    project: Project,
    packages: Vec<Package>,
    links: HashMap<i32, Link>,
    repositories_data: &'a RepositoriesDataSnapshot,
    redirect_from: Option<String>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn project_packages(
    Path(project_name): Path<String>,
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
) -> EndpointResult {
    let ctx = TemplateContext::new_without_params(Endpoint::ProjectPackages);

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

    let packages: Vec<Package> = sqlx::query_as(indoc! {"
        SELECT
            repo,
            subrepo,
            visiblename,
            origversion,
            rawversion,
            coalesce(maintainers, '{}'::text[]) AS maintainers,
            category,
            comment AS summary,
            coalesce(licenses, '{}'::text[]) AS licenses,
            version,
            versionclass AS status,
            flags,
            coalesce(
                (
                    SELECT json_agg(
                        json_array(
                            link->0,
                            link->1,
                            coalesce(link->2, 'null'::json)
                        )
                    ) FROM json_array_elements(links) AS link
                ),
                '[]'::json
            ) AS links
        FROM packages
        WHERE effname = $1
    "})
    .bind(&project_name)
    .fetch_all(&state.pool)
    .await?;

    let links: Vec<Link> = sqlx::query_as(indoc! {"
        SELECT
            id,
            url,
            last_checked,
            ipv4_success,
            ipv4_permanent_redirect_target IS NOT NULL AS has_ipv4_permanent_redirect,
            ipv6_success,
            ipv6_permanent_redirect_target IS NOT NULL AS has_ipv6_permanent_redirect
        FROM links WHERE id = ANY($1)
    "})
    .bind(
        packages
            .iter()
            .flat_map(|package| package.links.iter().map(|(_, link_id, _)| link_id))
            .unique()
            .collect::<Vec<_>>(),
    )
    .fetch_all(&state.pool)
    .await?;

    let mut packages_by_repo = packages
        .into_iter()
        .into_group_map_by(|package| package.repo.clone());

    let repositories_data = state.repository_data_cache.snapshot();

    // XXX: do we really want descending sort by version here?
    let packages: Vec<_> = repositories_data
        .active_repositories()
        .flat_map(|repository_data| {
            packages_by_repo
                .remove(repository_data.name.as_str())
                .map(|mut packages| {
                    by_name_asc_version_desc::sort(&mut packages);
                    packages
                })
                .unwrap_or_default()
        })
        .collect();

    let links: HashMap<_, _> = links.into_iter().map(|link| (link.id, link)).collect();

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            project_name: &project_name,
            project,
            packages,
            links,
            repositories_data: &repositories_data,
            redirect_from,
        }
        .render()?,
    )
        .into_response())
}
