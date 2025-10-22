// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use indoc::indoc;
use metrics::histogram;
use serde::Deserialize;
use sqlx::FromRow;

use repology_common::{PackageFlags, PackageStatus};

use crate::badges::{
    Cell, CellAlignment, DEFAULT_THEME, SpecialVersionStatus, badge_color_for_package_status,
    render_generic_badge,
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
    #[serde(rename = "minversion")]
    pub min_version: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub allow_ignored: bool,
    #[serde(default)]
    #[serde(rename = "columns")]
    pub columns_count: usize,

    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub exclude_unsupported: bool,

    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_seq")]
    pub exclude_sources: HashSet<SourceType>,
}

#[derive(FromRow)]
struct Package {
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

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn badge_vertical_allrepos(
    Path(project_name): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<QueryParams>,
) -> EndpointResult {
    let packages: Vec<Package> = sqlx::query_as(indoc! {"
        SELECT
            version,
            versionclass AS status,
            flags,
            repo AS repository_name
        FROM packages
        WHERE effname = $1;
    "})
    .bind(project_name)
    .fetch_all(&state.pool)
    .await?;

    let theme = &DEFAULT_THEME;

    let package_per_repository =
        pick_representative_package_per_repository(&packages, query.allow_ignored);

    let mut cells: Vec<Vec<Cell>> = vec![];

    for repository_data in state
        .repository_data_cache
        .snapshot()
        .active_repositories()
        .filter(|repository_data| !is_repository_filtered(repository_data, &query))
    {
        if let Some(&package) = package_per_repository.get(&repository_data.name) {
            let extra_status = query
                .min_version
                .as_ref()
                .is_some_and(|min_version| package_version(package) < min_version)
                .then_some(SpecialVersionStatus::LowerThanUserGivenThreshold);

            let color = badge_color_for_package_status(package.status, theme, extra_status);

            cells.push(vec![
                Cell::new(&repository_data.title).align(CellAlignment::Right),
                Cell::new(&package.version)
                    .color(color)
                    .truncate(13)
                    .min_width(60),
            ]);
        }
    }

    histogram!("repology_webapp_badge_vertical_allrepos_size").record(cells.len() as f64);

    let caption = query.caption.as_ref().map_or_else(
        || {
            if cells.is_empty() {
                Some("No known packages")
            } else {
                Some("Packaging status")
            }
        },
        |caption| {
            if caption.is_empty() {
                None
            } else {
                Some(caption)
            }
        },
    );

    let num_columns = query.columns_count.min(cells.len());

    if num_columns > 1 {
        let num_rows = cells.len().div_ceil(num_columns);

        let mut input_iter = cells.into_iter();
        cells = (0..num_rows)
            .map(|_| Vec::<Cell>::with_capacity(num_columns * 2))
            .collect();

        for i in 0..num_columns * num_rows {
            cells[i % num_rows].extend(input_iter.next().unwrap_or_default().into_iter());
        }
    };

    let body = render_generic_badge(&cells, caption, 0, &state.font_measurer, theme)?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::IMAGE_SVG.as_ref()),
        )],
        body,
    )
        .into_response())
}
