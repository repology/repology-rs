// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use indoc::indoc;
use serde::Deserialize;
use sqlx::FromRow;

use libversion::VersionStr;

use repology_common::{PackageFlags, PackageStatus};

use crate::badges::{
    Cell, CellAlignment, SpecialVersionStatus, badge_color_for_package_status, render_generic_badge,
};
use crate::package::processing::pick_representative_package_per_repository;
use crate::package::traits::{
    PackageWithFlags, PackageWithRepositoryName, PackageWithStatus, PackageWithVersion,
};
use crate::package::version::package_version;
use crate::repository_data::{RepositoryData, SourceType};
use crate::result::EndpointResult;
use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(rename = "header")]
    pub caption: Option<String>,
    #[serde(deserialize_with = "crate::query::deserialize_seq")]
    pub projects: Vec<String>,
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_seq")]
    pub repos: HashSet<String>,
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub require_all: bool,

    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub exclude_unsupported: bool,

    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_seq")]
    pub exclude_sources: HashSet<SourceType>,
}

#[derive(FromRow)]
struct Package {
    effname: String,
    version: String,
    status: PackageStatus,
    flags: i32,
    repository_name: String,
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
impl PackageWithRepositoryName for Package {
    fn repository_name(&self) -> &str {
        &self.repository_name
    }
}

fn is_repository_filtered(repository_data: &RepositoryData, query: &QueryParams) -> bool {
    if query.exclude_unsupported
        && repository_data
            .eol_date
            .is_some_and(|eol_date| eol_date < chrono::Utc::now().date_naive())
    {
        return true;
    }

    if !query.exclude_sources.is_empty()
        && query.exclude_sources.contains(&repository_data.source_type)
    {
        return true;
    }

    false
}

enum VersionRestriction<'a> {
    Greater(&'a str),
    GreaterOrEqual(&'a str),
    Lesser(&'a str),
    LesserOrEqual(&'a str),
    None,
}

impl VersionRestriction<'_> {
    pub fn is_passing(&self, version: &VersionStr) -> bool {
        match self {
            VersionRestriction::Greater(boundary) => version > boundary,
            VersionRestriction::GreaterOrEqual(boundary) => version >= boundary,
            VersionRestriction::Lesser(boundary) => version < boundary,
            VersionRestriction::LesserOrEqual(boundary) => version <= boundary,
            VersionRestriction::None => true,
        }
    }
}

/// Split an inequality expression.
///
/// Split a string containing an inequality expression such as `foo>=1` into
/// three parts - left hand side, operator, and right hand side. Allowed
/// operators are `>=`, `>`, `<=`, and `<`.
fn split_inequality(text: &str) -> Option<(&str, &str, &str)> {
    if let Some(op_pos) = text.find(['<', '>']) {
        let op_len: usize = if text.as_bytes().get(op_pos + 1) == Some(&b'=') {
            2
        } else {
            1
        };
        Some((
            &text[0..op_pos],
            &text[op_pos..op_pos + op_len],
            &text[op_pos + op_len..],
        ))
    } else {
        None
    }
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn badge_versions_matrix(
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let mut project_names: Vec<&str> = Vec::with_capacity(query.projects.len());
    let mut project_version_restrictions: Vec<VersionRestriction> =
        Vec::with_capacity(query.projects.len());

    for project in &query.projects {
        if let Some((project_name, operator, version)) = split_inequality(project) {
            project_names.push(project_name);
            project_version_restrictions.push(match operator {
                ">" => VersionRestriction::Greater(version),
                ">=" => VersionRestriction::GreaterOrEqual(version),
                "<" => VersionRestriction::Lesser(version),
                "<=" => VersionRestriction::LesserOrEqual(version),
                _ => unreachable!(),
            });
        } else {
            project_names.push(project);
            project_version_restrictions.push(VersionRestriction::None);
        }
    }

    let packages: Vec<Package> = sqlx::query_as(indoc! {"
        SELECT
            effname,
            version,
            versionclass AS status,
            flags,
            repo AS repository_name
        FROM packages
        WHERE effname = ANY($1)
    "})
    .bind(&project_names)
    .fetch_all(&state.pool)
    .await?;

    // group packages per project
    let mut packages_per_project: HashMap<String, Vec<Package>> =
        HashMap::with_capacity(project_names.len());

    packages.into_iter().for_each(|package| {
        packages_per_project
            .entry(package.effname.clone())
            .or_default()
            .push(package)
    });

    // pick a package per project & repository
    let packages_per_project: HashMap<&str, _> = packages_per_project
        .iter()
        .map(|(project_name, packages)| {
            (
                project_name.as_str(),
                pick_representative_package_per_repository(packages, false),
            )
        })
        .collect();

    // header row
    let mut cells: Vec<Vec<Cell>> = vec![vec![Cell::empty()]];
    project_names
        .iter()
        .map(|project_name| Cell::new(project_name))
        .collect_into(&mut cells[0]);

    // per-repository rows
    for repository_data in state
        .repository_data_cache
        .snapshot()
        .active_repositories()
        .filter(|repository_data| !is_repository_filtered(repository_data, &query))
    {
        let mut row = vec![Cell::new(&repository_data.title).align(CellAlignment::Right)];
        let mut filled_cells: usize = 0;
        let mut empty_cells: usize = 0;

        for (project_name, version_restriction) in project_names
            .iter()
            .zip(project_version_restrictions.iter())
        {
            if let Some(&package) = packages_per_project
                .get(project_name)
                .and_then(|per_repository| per_repository.get(&repository_data.name))
            {
                let extra_status = Some(SpecialVersionStatus::LowerThanUserGivenThreshold)
                    .filter(|_| !version_restriction.is_passing(&package_version(package)));
                let color = badge_color_for_package_status(package.status, extra_status);

                row.push(
                    Cell::new(&package.version)
                        .color(color)
                        .truncate(13)
                        .min_width(60),
                );
                filled_cells += 1;
            } else {
                row.push(Cell::new("-"));
                empty_cells += 1;
            }
        }

        if filled_cells == 0 && !query.repos.contains(&repository_data.name) {
            // we don't need a row for repository which contains none of relevant projects,
            // unless the repository was explicitly requested
            continue;
        }

        if query.require_all && empty_cells != 0 {
            // if such mode was requested, we don't need a row for repository which does not
            // contain all relevant projects
            continue;
        }

        cells.push(row);
    }

    let body = render_generic_badge(&cells, query.caption.as_deref(), 0, &state.font_measurer)?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::IMAGE_SVG.as_ref()),
        )],
        body,
    )
        .into_response())
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_split_ineqiality() {
        assert_eq!(split_inequality("a>b"), Some(("a", ">", "b")));
        assert_eq!(split_inequality("a>=b"), Some(("a", ">=", "b")));
        assert_eq!(split_inequality("a<b"), Some(("a", "<", "b")));
        assert_eq!(split_inequality("a<=b"), Some(("a", "<=", "b")));
        assert_eq!(split_inequality(">"), Some(("", ">", "")));
        assert_eq!(split_inequality(">="), Some(("", ">=", "")));
        assert_eq!(split_inequality("<"), Some(("", "<", "")));
        assert_eq!(split_inequality("<="), Some(("", "<=", "")));
        assert_eq!(split_inequality(""), None);
        assert_eq!(split_inequality("abc"), None);
        assert_eq!(split_inequality("a=b"), None);
    }
}
