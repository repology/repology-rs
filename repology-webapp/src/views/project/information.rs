// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod accumulators;
mod emails;
mod slices;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;
use indoc::indoc;
use sqlx::FromRow;

use repology_common::{LinkType, PackageFlags, PackageStatus};

use crate::endpoints::Endpoint;
use crate::package::summarization::DisplayVersion;
use crate::package::traits::{PackageWithFlags, PackageWithStatus, PackageWithVersion};
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

use super::common::Project;
use super::nonexistent::nonexisting_project;

use self::accumulators::SlicesAccumulator;
use self::slices::*;

#[derive(FromRow)]
pub struct Package {
    pub repo: String,
    pub family: String,
    pub projectname_seed: String,
    pub version: String,
    pub status: PackageStatus,
    pub flags: i32,
    pub summary: Option<String>,
    pub maintainers: Vec<String>,
    pub licenses: Vec<String>,
    pub categories: Vec<String>,
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
impl PackageWithStatus for Package {
    fn status(&self) -> PackageStatus {
        self.status
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
#[template(path = "project/information.html")]
struct TemplateParams<'a> {
    ctx: TemplateContext,
    project_name: String,
    project: Option<Project>,
    slices: Slices<'a>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn project_information(
    Path(project_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new_without_params(Endpoint::ProjectInformation);

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
        return nonexisting_project(&*state, ctx, project_name, project).await;
    }

    // TODO: try fetching project and packages in parallel tasks, see
    // if this affects latency
    let packages: Vec<Package> = sqlx::query_as(indoc! {"
        SELECT
            repo,
            family,
            projectname_seed,
            version,
            versionclass AS status,
            flags,
            comment AS summary,
            coalesce(maintainers, '{}'::text[]) AS maintainers,
            coalesce(licenses, '{}'::text[]) AS licenses,
            CASE WHEN category IS NULL THEN '{}'::text[] ELSE ARRAY[category] END AS categories,
            coalesce(
                (
                    SELECT json_agg(
                        json_array(
                            link->0,
                            link->1,
                            coalesce(link->2, 'null'::json)
                        )
                    ) FROM json_array_elements(links) as link
                ),
                '[]'::json
            ) AS links
        FROM packages
        WHERE effname = $1
    "})
    .bind(&project_name)
    .fetch_all(&state.pool)
    .await?;

    let mut accum: SlicesAccumulator = Default::default();

    // XXX: somewhat similar to packages_to_categorized_display_versions_per_project, merge code
    for package in &packages {
        accum.repositories.insert(&package.repo);
        accum.add_string_slice(
            StringSliceType::Name,
            &package.projectname_seed,
            &package.family,
        );
        {
            let mut version = DisplayVersion::from_package(package);
            if package.status == PackageStatus::Legacy {
                version.status = PackageStatus::Outdated;
            }
            accum.versions.push((version, package.family.as_str()));
        }

        if let Some(summary) = &package.summary {
            accum.add_string_slice(StringSliceType::Summary, &summary, &package.family);
        }
        for maintainer in &package.maintainers {
            accum.add_string_slice(StringSliceType::Maintainer, &maintainer, &package.family);
            accum.maintainer_emails.add(&maintainer);
        }
        for category in &package.categories {
            accum.add_string_slice(StringSliceType::Category, &category, &package.family);
        }
        for license in &package.licenses {
            accum.add_string_slice(StringSliceType::License, &license, &package.family);
        }

        // within a given package, we don't want to duplicate e.g. Recipe and RecipeRaw links,
        // so skip links of type FooRaw if we have links of type Foo
        let link_types_to_skip: HashSet<LinkType> = package
            .links
            .iter()
            .filter_map(|(link_type, _, _)| link_type.raw_counterpart())
            .collect();

        for (link_type, link_id, link_fragment) in package.links.as_ref() {
            if link_types_to_skip.contains(link_type) {
                continue;
            };

            use LinkType::*;
            if let Some(link_slice_type) = match link_type {
                UpstreamHomepage | ProjectHomepage => Some(LinkSliceType::Homepage),
                UpstreamDownload | ProjectDownload | UpstreamDownloadPage => {
                    Some(LinkSliceType::Download)
                }
                UpstreamIssueTracker => Some(LinkSliceType::Issues),
                UpstreamRepository => Some(LinkSliceType::Repository),
                UpstreamDocumentation => Some(LinkSliceType::Documentation),
                PackageHomepage => Some(LinkSliceType::Package),
                PackageRecipe | PackageRecipeRaw => Some(LinkSliceType::Recipe),
                PackagePatch | PackagePatchRaw => Some(LinkSliceType::Patch),
                PackageBuildLog | PackageBuildLogs | PackageBuildLogRaw => {
                    Some(LinkSliceType::BuildLog)
                }
                _ => None,
            } {
                accum.add_link_slice(
                    link_slice_type,
                    *link_id,
                    link_fragment.as_deref(),
                    &package.family,
                );
            }
        }
    }

    let links: Vec<Link> = sqlx::query_as(indoc! {"
        SELECT
            id,
            url,
            last_checked,
            ipv4_success,
            ipv4_permanent_redirect_target IS NOT NULL AS has_ipv4_permanent_redirect,
            ipv6_success,
            ipv6_permanent_redirect_target IS NOT NULL AS has_ipv6_permanent_redirect
        FROM links
        WHERE id = ANY($1)
    "})
    .bind(&accum.get_all_link_ids())
    .fetch_all(&state.pool)
    .await?;

    let links: HashMap<i32, Link> = links.into_iter().map(|link| (link.id, link)).collect();
    let repositories_data = state.repository_data_cache.get_all_active();

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            project_name,
            project,
            slices: accum.finalize(&links, &repositories_data),
        }
        .render()?,
    )
        .into_response())
}
