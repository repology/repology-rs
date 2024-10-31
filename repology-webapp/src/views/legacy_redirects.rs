// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::extract::{Path, Query};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;

use crate::endpoints::Endpoint;
use crate::result::EndpointResult;
use crate::template_context::TemplateContext;

pub async fn legacy_badge_version_only_for_repo(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
) -> EndpointResult {
    Ok((
        StatusCode::MOVED_PERMANENTLY,
        [(
            header::LOCATION,
            HeaderValue::from_maybe_shared(
                TemplateContext::new(Endpoint::BadgeVersionForRepo, gen_path, gen_query)
                    .url_for_self(&[])?,
            )?,
        )],
    )
        .into_response())
}