// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::Template;
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;

use crate::endpoints::{Endpoint, MyEndpoint};
use crate::result::EndpointResult;

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn opensearch_project(endpoint: MyEndpoint) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "opensearch/project.xml")]
    struct TemplateParams<'a> {
        endpoint: &'a MyEndpoint,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        )],
        TemplateParams {
            endpoint: &endpoint,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn opensearch_maintainer(endpoint: MyEndpoint) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "opensearch/maintainer.xml")]
    struct TemplateParams<'a> {
        endpoint: &'a MyEndpoint,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        )],
        TemplateParams {
            endpoint: &endpoint,
        }
        .render()?,
    )
        .into_response())
}
