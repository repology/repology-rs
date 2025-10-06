// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use indoc::indoc;
use itertools::Itertools;
use serde::Deserialize;
use sqlx::FromRow;

use libversion::AsVersionWithFlags;

use repology_common::PackageFlags;

use crate::badges::{Cell, DEFAULT_THEME, render_generic_badge};
use crate::package::traits::{PackageWithFlags, PackageWithVersion};
use crate::package::version::package_version;
use crate::result::EndpointResult;
use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(rename = "header")]
    pub caption: Option<String>,
}

#[derive(FromRow)]
struct Package {
    version: String,
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

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn badge_latest_versions(
    Path(project_name): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<QueryParams>,
) -> EndpointResult {
    let packages: Vec<Package> = sqlx::query_as(indoc! {"
        SELECT
            version,
            flags
        FROM packages
        WHERE effname = $1 AND versionclass IN (1, 4, 5)  -- Newest, Unique, Devel
    "})
    .bind(project_name)
    .fetch_all(&state.pool)
    .await?;

    let versions = packages
        .iter()
        .map(|package| package_version(package))
        .sorted_by(|a, b| {
            // version desc → version string length desc → version string lexographical
            a.cmp(b)
                .then_with(|| a.version().len().cmp(&b.version().len()))
                .reverse()
                .then_with(|| a.version().cmp(b.version()))
        })
        .dedup_by(|a, b| a.version() == b.version())
        .collect::<Vec<_>>();

    let (default_caption, text) = match versions.len() {
        0 => ("latest packaged version", Cow::from("-")),
        1 => ("latest packaged version", Cow::from(versions[0].version())),
        _ => (
            "latest packaged versions",
            Cow::from(versions.iter().map(|version| version.version()).join(", ")),
        ),
    };

    let theme = &DEFAULT_THEME;

    let caption_cell = Cell::new(
        query
            .caption
            .as_ref()
            .map_or(default_caption, String::as_str),
    )
    .collapsible(true);
    let version_cell = Cell::new(&text).color(theme.color_nice);

    let body = render_generic_badge(
        &[vec![caption_cell, version_cell]],
        None,
        0,
        &state.font_measurer,
        theme,
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
