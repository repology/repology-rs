// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::Template;
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;

use crate::endpoints::{Endpoint, MyEndpoint};
use crate::result::EndpointResult;
use crate::template_context::TemplateContext;

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn opensearch_project(endpoint: MyEndpoint) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "opensearch/project.xml")]
    struct TemplateParams<'a> {
        ctx: TemplateContext,
        endpoint: &'a MyEndpoint,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::OpensearchProject);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        )],
        TemplateParams {
            ctx,
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
        ctx: TemplateContext,
        endpoint: &'a MyEndpoint,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::OpensearchMaintainer);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        )],
        TemplateParams {
            ctx,
            endpoint: &endpoint,
        }
        .render()?,
    )
        .into_response())
}
