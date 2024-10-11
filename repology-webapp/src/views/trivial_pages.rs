// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::Template;
use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;

use crate::endpoints::Endpoint;
use crate::result::EndpointResult;
use crate::template_context::TemplateContext;

#[tracing::instrument()]
pub async fn news() -> EndpointResult {
    #[derive(Template)]
    #[template(path = "news.html")]
    struct TemplateParams {
        ctx: TemplateContext,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::News);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams { ctx }.render()?,
    )
        .into_response())
}

#[tracing::instrument()]
pub async fn docs() -> EndpointResult {
    #[derive(Template)]
    #[template(path = "docs/index.html")]
    struct TemplateParams {
        ctx: TemplateContext,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::Docs);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams { ctx }.render()?,
    )
        .into_response())
}

#[tracing::instrument()]
pub async fn docs_about() -> EndpointResult {
    #[derive(Template)]
    #[template(path = "docs/about.html")]
    struct TemplateParams {
        ctx: TemplateContext,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::DocsAbout);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams { ctx }.render()?,
    )
        .into_response())
}

#[tracing::instrument()]
pub async fn docs_bots() -> EndpointResult {
    #[derive(Template)]
    #[template(path = "docs/bots.html")]
    struct TemplateParams {
        ctx: TemplateContext,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::DocsBots);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams { ctx }.render()?,
    )
        .into_response())
}

#[tracing::instrument()]
pub async fn docs_not_supported() -> EndpointResult {
    #[derive(Template)]
    #[template(path = "docs/not_supported.html")]
    struct TemplateParams {
        ctx: TemplateContext,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::DocsNotSupported);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams { ctx }.render()?,
    )
        .into_response())
}

#[tracing::instrument()]
pub async fn docs_requirements() -> EndpointResult {
    #[derive(Template)]
    #[template(path = "docs/requirements.html")]
    struct TemplateParams {
        ctx: TemplateContext,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::DocsRequirements);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams { ctx }.render()?,
    )
        .into_response())
}

#[tracing::instrument()]
pub async fn tools() -> EndpointResult {
    #[derive(Template)]
    #[template(path = "tools/index.html")]
    struct TemplateParams {
        ctx: TemplateContext,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::Tools);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams { ctx }.render()?,
    )
        .into_response())
}

#[tracing::instrument()]
pub async fn api_v1() -> EndpointResult {
    #[derive(Template)]
    #[template(path = "api.html")]
    struct TemplateParams {
        ctx: TemplateContext,
        per_page: u32,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::ApiV1);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            per_page: crate::constants::PROJECTS_PER_PAGE,
        }
        .render()?,
    )
        .into_response())
}
