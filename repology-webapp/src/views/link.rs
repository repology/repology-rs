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

use repology_common::LinkStatus;

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
struct DbLink {
    url: String,
    first_extracted: DateTime<Utc>,
    last_checked: Option<DateTime<Utc>>,
    last_success: Option<DateTime<Utc>>,
    last_failure: Option<DateTime<Utc>>,
    ipv4_status_code: Option<i16>,
    ipv4_permanent_redirect_target: Option<String>,
    ipv6_status_code: Option<i16>,
    ipv6_permanent_redirect_target: Option<String>,
}

struct Link {
    url: String,
    first_extracted: DateTime<Utc>,
    last_checked: Option<DateTime<Utc>>,
    last_success: Option<DateTime<Utc>>,
    last_failure: Option<DateTime<Utc>>,
    ipv4_status: Option<LinkStatus>,
    ipv4_permanent_redirect_target: Option<String>,
    ipv6_status: Option<LinkStatus>,
    ipv6_permanent_redirect_target: Option<String>,
}

impl TryFrom<DbLink> for Link {
    type Error = anyhow::Error;

    fn try_from(link: DbLink) -> Result<Self, Self::Error> {
        Ok(Self {
            url: link.url,
            first_extracted: link.first_extracted,
            last_checked: link.last_checked,
            last_success: link.last_success,
            last_failure: link.last_failure,
            ipv4_status: link
                .ipv4_status_code
                .map(|code| LinkStatus::try_from(code))
                .transpose()?,
            ipv4_permanent_redirect_target: link.ipv4_permanent_redirect_target,
            ipv6_status: link
                .ipv6_status_code
                .map(|code| LinkStatus::try_from(code))
                .transpose()?,
            ipv6_permanent_redirect_target: link.ipv6_permanent_redirect_target,
        })
    }
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn link(Path(url): Path<String>, State(state): State<Arc<AppState>>) -> EndpointResult {
    let ctx = TemplateContext::new_without_params(Endpoint::Link);

    let link: Option<DbLink> = sqlx::query_as(indoc! {r#"
        SELECT
            url,
            first_extracted,
            last_checked,
            last_success,
            last_failure,
            ipv4_status_code,
            ipv4_permanent_redirect_target,
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
        TemplateParams {
            ctx,
            link: link.try_into()?,
        }
        .render()?,
    )
        .into_response())
}
