// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

// triggered inside template
#![allow(clippy::manual_range_contains)]

use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use sqlx::FromRow;

use crate::endpoints::Endpoint;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

#[derive(Template)]
#[template(path = "link.html")]
struct TemplateParams {
    ctx: TemplateContext,
    link: Link,
}

#[derive(FromRow)]
struct Link {
    url: String,
    first_extracted: DateTime<Utc>,
    last_checked: Option<DateTime<Utc>>,

    ipv4_last_success: Option<DateTime<Utc>>,
    ipv4_last_failure: Option<DateTime<Utc>>,
    ipv4_success: Option<bool>,
    ipv4_status_code: Option<i16>,
    ipv4_permanent_redirect_target: Option<String>,

    ipv6_last_success: Option<DateTime<Utc>>,
    ipv6_last_failure: Option<DateTime<Utc>>,
    ipv6_success: Option<bool>,
    ipv6_status_code: Option<i16>,
    ipv6_permanent_redirect_target: Option<String>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn link(Path(url): Path<String>, State(state): State<Arc<AppState>>) -> EndpointResult {
    let ctx = TemplateContext::new_without_params(Endpoint::Link);

    let link: Option<Link> = sqlx::query_as(indoc! {r#"
        SELECT
            url,
            first_extracted,
            last_checked,

            ipv4_last_success,
            ipv4_last_failure,
            ipv4_success,
            ipv4_status_code,
            ipv4_permanent_redirect_target,

            ipv6_last_success,
            ipv6_last_failure,
            ipv6_success,
            ipv6_status_code,
            ipv6_permanent_redirect_target
        FROM links
        WHERE url = $1
    "#})
    .bind(&url)
    .fetch_optional(&state.pool)
    .await?;

    let Some(link) = link else {
        return Ok((StatusCode::NOT_FOUND, "link not found".to_owned()).into_response());
    };

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams { ctx, link }.render()?,
    )
        .into_response())
}
