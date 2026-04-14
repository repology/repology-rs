// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::Template;
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;

use crate::endpoints::MyEndpoint;
use crate::result::EndpointResult;

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn news(endpoint: MyEndpoint) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "news.html")]
    struct TemplateParams<'a> {
        endpoint: &'a MyEndpoint,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            endpoint: &endpoint,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn docs(endpoint: MyEndpoint) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "docs/index.html")]
    struct TemplateParams<'a> {
        endpoint: &'a MyEndpoint,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            endpoint: &endpoint,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn docs_about(endpoint: MyEndpoint) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "docs/about.html")]
    struct TemplateParams<'a> {
        endpoint: &'a MyEndpoint,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            endpoint: &endpoint,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn docs_bots(endpoint: MyEndpoint) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "docs/bots.html")]
    struct TemplateParams<'a> {
        endpoint: &'a MyEndpoint,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            endpoint: &endpoint,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn docs_not_supported(endpoint: MyEndpoint) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "docs/not_supported.html")]
    struct TemplateParams<'a> {
        endpoint: &'a MyEndpoint,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            endpoint: &endpoint,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn docs_requirements(endpoint: MyEndpoint) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "docs/requirements.html")]
    struct TemplateParams<'a> {
        endpoint: &'a MyEndpoint,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            endpoint: &endpoint,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn tools(endpoint: MyEndpoint) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "tools/index.html")]
    struct TemplateParams<'a> {
        endpoint: &'a MyEndpoint,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            endpoint: &endpoint,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn api_v1(endpoint: MyEndpoint) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "api.html")]
    struct TemplateParams<'a> {
        endpoint: &'a MyEndpoint,
        per_page: usize,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            endpoint: &endpoint,
            per_page: crate::constants::PROJECTS_PER_PAGE,
        }
        .render()?,
    )
        .into_response())
}
