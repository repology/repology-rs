// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(warnings, unused)]

use std::collections::HashSet;

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use indoc::indoc;
use metrics::histogram;
use serde::Deserialize;
use sqlx::FromRow;

use repology_common::{PackageFlags, PackageStatus};

use crate::badges::{
    badge_color_for_package_status, render_generic_badge, Cell, CellAlignment, SpecialVersionStatus,
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
}

#[derive(FromRow, Default)]
pub struct RepositoryStatistics {
    pub num_projects: i32,
    pub num_projects_comparable: i32,
    pub num_projects_newest: i32,
    pub num_projects_outdated: i32,
    pub num_projects_vulnerable: i32,
    pub num_projects_problematic: i32,
    pub num_maintainers: i32,
}

fn format_percentage(divident: i32, divisor: i32) -> String {
    if divisor == 0 {
        // TODO: switch to Cow here
        "-".to_string()
    } else {
        format!("{:.2}%", 100.0 * divident as f64 / divisor as f64)
    }
}

#[tracing::instrument(skip(state))]
pub async fn badge_repository_big(
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<AppState>,
) -> EndpointResult {
    let repository_name = if let Some(repository_name) = repository_name.strip_suffix(".svg") {
        repository_name
    } else {
        return Ok((StatusCode::NOT_FOUND, "path must end with .svg".to_owned()).into_response());
    };

    let statistics: RepositoryStatistics = sqlx::query_as(indoc! {r#"
        SELECT
            num_metapackages AS num_projects,
            num_metapackages_comparable AS num_projects_comparable,
            num_metapackages_newest AS num_projects_newest,
            num_metapackages_outdated AS num_projects_outdated,
            num_metapackages_vulnerable AS num_projects_vulnerable,
            num_metapackages_problematic AS num_projects_problematic,
            num_maintainers
        FROM repositories
        WHERE name = $1
    "#})
    .bind(repository_name)
    .fetch_optional(&state.pool)
    .await?
    .unwrap_or_default();

    let caption = query
        .caption
        .as_deref()
        .map_or(Some("Repository status"), |caption| {
            Some(caption).filter(|caption| !caption.is_empty())
        });

    let mut cells: Vec<Vec<Cell>> = vec![vec![
        Cell::new("Projects total").align(CellAlignment::Right),
        Cell::new(&format!("{}", statistics.num_projects)),
    ]];

    if statistics.num_projects > 0 {
        // will need third column for percentages
        cells[0].push(Cell::empty());

        let color = badge_color_for_package_status(PackageStatus::Newest, None);
        cells.push(vec![
            Cell::new("Up to date").align(CellAlignment::Right),
            Cell::new(&format!("{}", statistics.num_projects_newest)).color(color),
            Cell::new(&format_percentage(
                statistics.num_projects_newest,
                statistics.num_projects_comparable,
            ))
            .color(color),
        ]);

        let color = badge_color_for_package_status(PackageStatus::Outdated, None);
        cells.push(vec![
            Cell::new("Outdated").align(CellAlignment::Right),
            Cell::new(&format!("{}", statistics.num_projects_outdated)).color(color),
            Cell::new(&format_percentage(
                statistics.num_projects_outdated,
                statistics.num_projects_comparable,
            ))
            .color(color),
        ]);

        let color = "#e00000";
        cells.push(vec![
            Cell::new("Vulnerable").align(CellAlignment::Right),
            Cell::new(&format!("{}", statistics.num_projects_vulnerable)).color(color),
            Cell::new(&format_percentage(
                statistics.num_projects_vulnerable,
                statistics.num_projects,
            ))
            .color(color),
        ]);

        let color = "#9f9f9f";
        cells.push(vec![
            Cell::new("Bad versions").align(CellAlignment::Right),
            Cell::new(&format!("{}", statistics.num_projects_problematic)).color(color),
            Cell::new(&format_percentage(
                statistics.num_projects_problematic,
                statistics.num_projects,
            ))
            .color(color),
        ]);

        if statistics.num_maintainers > 0 {
            cells.push(vec![
                Cell::new("Maintainers").align(CellAlignment::Right),
                Cell::new(&format!("{}", statistics.num_maintainers)),
                Cell::empty(),
            ]);
        }
    }

    let body = render_generic_badge(&cells, caption, 0, &state.font_measurer)?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::IMAGE_SVG.as_ref()),
        )],
        body,
    )
        .into_response())
}
