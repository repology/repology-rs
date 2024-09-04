// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use serde::Deserialize;
use sqlx::FromRow;
use std::collections::HashSet;

use repology_common::{PackageFlags, PackageStatus};

use crate::badges::{
    badge_color_for_package_status, render_generic_badge, Cell, CellAlignment, SpecialVersionStatus,
};
use crate::package::processing::pick_representative_package_per_repository;
use crate::package::traits::{
    PackageWithFlags, PackageWithRepositoryName, PackageWithStatus, PackageWithVersion,
};
use crate::package::version::package_version;
use crate::repometadata::{RepositoryMetadata, SourceType};
use crate::result::EndpointResult;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct QueryParams {
    #[serde(rename = "header")]
    pub caption: Option<String>,
    #[serde(rename = "minversion")]
    pub min_version: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub allow_ignored: bool,
    #[serde(default)]
    pub columns_count: usize,

    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub exclude_unsupported: bool,

    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_seq")]
    pub exclude_sources: HashSet<SourceType>,
}

#[derive(FromRow)]
pub struct Package {
    pub version: String,
    pub status: PackageStatus,
    pub flags: i32,
    pub repository_name: String,
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

fn is_repository_filtered(repository_metadata: &RepositoryMetadata, query: &QueryParams) -> bool {
    if query.exclude_unsupported
        && repository_metadata
            .eol_date
            .is_some_and(|eol_date| eol_date < chrono::Utc::now().date_naive())
    {
        return true;
    }

    if !query.exclude_sources.is_empty()
        && query
            .exclude_sources
            .contains(&repository_metadata.source_type)
    {
        return true;
    }

    false
}

pub async fn badge_vertical_allrepos(
    Path(project_name): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<QueryParams>,
) -> EndpointResult {
    let project_name = if let Some(project_name) = project_name.strip_suffix(".svg") {
        project_name
    } else {
        return Ok((StatusCode::NOT_FOUND, "path must end with .svg".to_owned()).into_response());
    };

    let packages: Vec<Package> = sqlx::query_as(
        r#"
        SELECT
            version,
            versionclass AS status,
            flags,
            repo AS repository_name
        FROM packages
        WHERE effname = $1;
        "#,
    )
    .bind(project_name)
    .fetch_all(&state.pool)
    .await?;

    let package_per_repository =
        pick_representative_package_per_repository(&packages, query.allow_ignored);

    let mut cells: Vec<Vec<Cell>> = vec![];

    for repository_metadata in state
        .repository_metadata_cache
        .get_all_active()
        .await
        .into_iter()
        .filter(|repository_metadata| !is_repository_filtered(repository_metadata, &query))
    {
        if let Some(&package) = package_per_repository.get(&repository_metadata.name) {
            let extra_status = query
                .min_version
                .as_ref()
                .is_some_and(|min_version| package_version(package) < min_version)
                .then_some(SpecialVersionStatus::LowerThanUserGivenThreshold);

            let color = badge_color_for_package_status(package.status, extra_status);

            cells.push(vec![
                Cell::new(&repository_metadata.title).align(CellAlignment::Right),
                Cell::new(&package.version)
                    .color(color)
                    .truncate(13)
                    .min_width(60),
            ]);
        }
    }

    let caption = query.caption.as_ref().map_or(
        if cells.is_empty() {
            "No known packages"
        } else {
            "Packaging status"
        },
        String::as_str,
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

    let body = render_generic_badge(&cells, Some(caption), 0, &state.font_measurer)?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::IMAGE_SVG.as_ref()),
        )],
        body,
    )
        .into_response())
}
