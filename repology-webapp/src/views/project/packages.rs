// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;
use indoc::indoc;
use itertools::Itertools;
use serde::Deserialize;
use sqlx::FromRow;

use repology_common::{LinkType, PackageFlags, PackageStatus};

use crate::endpoints::Endpoint;
use crate::package::ordering::by_name_asc_version_desc;
use crate::package::traits::{PackageWithDisplayName, PackageWithFlags, PackageWithVersion};
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
    project: Option<Project>,
    packages: Vec<Package>,
    links: HashMap<i32, Link>,
    repositories_data: Vec<RepositoryData>,
}

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub experimental_link_query: bool,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn project_packages(
    Path(project_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<AppState>,
) -> EndpointResult {
    let ctx = TemplateContext::new_without_params(Endpoint::ProjectPackages);

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

    let links: Vec<Link> = if query.experimental_link_query {
        let all_link_ids: Vec<_> = packages
            .iter()
            .flat_map(|package| package.links.iter().map(|(_, link_id, _)| link_id))
            .unique()
            .collect();

        sqlx::query_as(indoc! {"
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
        .bind(&all_link_ids)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as(indoc! {"
            WITH link_ids AS (
                SELECT DISTINCT (json_array_elements(links)->>1)::integer AS id
                FROM packages
                WHERE effname = $1
            )
            SELECT
                id,
                url,
                last_checked,
                ipv4_success,
                ipv4_permanent_redirect_target IS NOT NULL AS has_ipv4_permanent_redirect,
                ipv6_success,
                ipv6_permanent_redirect_target IS NOT NULL AS has_ipv6_permanent_redirect
            FROM links INNER JOIN link_ids USING(id);
        "})
        .bind(&project_name)
        .fetch_all(&state.pool)
        .await?
    };

    let mut packages_by_repo = packages
        .into_iter()
        .into_group_map_by(|package| package.repo.clone());

    let repositories_data = state.repository_data_cache.get_all_active().await;

    // XXX: do we really want descending sort by version here?
    let packages: Vec<_> = repositories_data
        .iter()
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
            repositories_data,
        }
        .render()?,
    )
        .into_response())
}
