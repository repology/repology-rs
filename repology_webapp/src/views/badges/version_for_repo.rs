// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use serde::Deserialize;
use sqlx::FromRow;

use repology_common::{PackageFlags, PackageStatus};

use crate::badges::{
    badge_color_for_package_status, render_generic_badge, Cell, SpecialVersionStatus,
};
use crate::package::processing::pick_representative_package;
use crate::package::traits::{PackageWithFlags, PackageWithStatus, PackageWithVersion};
use crate::package::version::package_version;
use crate::result::EndpointResult;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct QueryParams {
    #[serde(rename = "header")]
    pub caption: Option<String>,
    #[serde(rename = "minversion")]
    pub min_version: Option<String>,
    pub allow_ignored: Option<bool>,
}

#[derive(FromRow)]
pub struct Package {
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
impl PackageWithStatus for Package {
    fn status(&self) -> PackageStatus {
        self.status
    }
}

pub async fn badge_version_for_repo(
    Path((repository_name, project_name)): Path<(String, String)>,
    State(state): State<AppState>,
    Query(query): Query<QueryParams>,
) -> EndpointResult {
    let project_name = if let Some(project_name) = project_name.strip_suffix(".svg") {
        project_name
    } else {
        return Ok((StatusCode::NOT_FOUND, "path must end with .svg".to_owned()).into_response());
    };

    let singular = if let Some(repository_metadata) = state
        .repository_metadata_cache
        .get_active(&repository_name)
        .await
    {
        repository_metadata.singular
    } else {
        return Ok((StatusCode::NOT_FOUND, "repository not found".to_owned()).into_response());
    };

    let packages: Vec<Package> = sqlx::query_as(
        r#"
        SELECT
            version,
            versionclass AS status,
            flags
        FROM packages
        WHERE repo = $1 AND effname = $2;
        "#,
    )
    .bind(repository_name)
    .bind(project_name)
    .fetch_all(&state.pool)
    .await?;

    let caption_cell =
        Cell::new(query.caption.as_ref().map_or(&singular, |caption| caption)).collapsible(true);
    let version_cell = if let Some(package) =
        pick_representative_package(&packages, query.allow_ignored.unwrap_or_default())
    {
        let extra_status = query
            .min_version
            .as_ref()
            .is_some_and(|min_version| package_version(package) < min_version)
            .then_some(SpecialVersionStatus::LowerThanUserGivenThreshold);
        let color = badge_color_for_package_status(package.status, extra_status);

        Cell::new(&package.version).color(color).truncate(20)
    } else {
        Cell::new("-")
    };

    let body = render_generic_badge(
        &[vec![caption_cell, version_cell]],
        None,
        0,
        &state.font_measurer,
    )?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::IMAGE_SVG.as_ref()),
        )],
        body,
    )
        .into_response())
}
