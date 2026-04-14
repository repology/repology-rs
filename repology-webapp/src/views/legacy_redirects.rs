// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::extract::{Path, Query};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;

use crate::endpoints::{Endpoint, MyEndpoint};
use crate::result::EndpointResult;

fn redirect(target: String) -> EndpointResult {
    Ok((
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, HeaderValue::from_maybe_shared(target)?)],
    )
        .into_response())
}

pub async fn legacy_badge_version_only_for_repo(endpoint: MyEndpoint) -> EndpointResult {
    redirect(
        Endpoint::BadgeVersionForRepo
            .url_for()
            .filled_from(endpoint.url_for_self())
            .build()?,
    )
}

pub async fn legacy_metapackage_versions(endpoint: MyEndpoint) -> EndpointResult {
    redirect(
        Endpoint::ProjectVersions
            .url_for()
            .filled_from(endpoint.url_for_self())
            .build()?,
    )
}

pub async fn legacy_metapackage_packages(endpoint: MyEndpoint) -> EndpointResult {
    redirect(
        Endpoint::ProjectPackages
            .url_for()
            .filled_from(endpoint.url_for_self())
            .build()?,
    )
}
