// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;

use crate::result::HandlerResult;
use crate::routes::{MyRoute, Route};

fn redirect(target: String) -> HandlerResult {
    Ok((
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, HeaderValue::from_maybe_shared(target)?)],
    )
        .into_response())
}

pub async fn legacy_badge_version_only_for_repo(my_route: MyRoute) -> HandlerResult {
    redirect(
        Route::BadgeVersionForRepo
            .url_for()
            .filled_from(my_route.url_for_self())?
            .build()?,
    )
}

pub async fn legacy_metapackage_versions(my_route: MyRoute) -> HandlerResult {
    redirect(
        Route::ProjectVersions
            .url_for()
            .filled_from(my_route.url_for_self())?
            .build()?,
    )
}

pub async fn legacy_metapackage_packages(my_route: MyRoute) -> HandlerResult {
    redirect(
        Route::ProjectPackages
            .url_for()
            .filled_from(my_route.url_for_self())?
            .build()?,
    )
}
