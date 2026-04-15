// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::Template;
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;

use crate::result::HandlerResult;
use crate::routes::MyRoute;

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn news(my_route: MyRoute) -> HandlerResult {
    #[derive(Template)]
    #[template(path = "news.html")]
    struct TemplateParams<'a> {
        my_route: &'a MyRoute,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            my_route: &my_route,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn docs(my_route: MyRoute) -> HandlerResult {
    #[derive(Template)]
    #[template(path = "docs/index.html")]
    struct TemplateParams<'a> {
        my_route: &'a MyRoute,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            my_route: &my_route,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn docs_about(my_route: MyRoute) -> HandlerResult {
    #[derive(Template)]
    #[template(path = "docs/about.html")]
    struct TemplateParams<'a> {
        my_route: &'a MyRoute,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            my_route: &my_route,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn docs_bots(my_route: MyRoute) -> HandlerResult {
    #[derive(Template)]
    #[template(path = "docs/bots.html")]
    struct TemplateParams<'a> {
        my_route: &'a MyRoute,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            my_route: &my_route,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn docs_not_supported(my_route: MyRoute) -> HandlerResult {
    #[derive(Template)]
    #[template(path = "docs/not_supported.html")]
    struct TemplateParams<'a> {
        my_route: &'a MyRoute,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            my_route: &my_route,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn docs_requirements(my_route: MyRoute) -> HandlerResult {
    #[derive(Template)]
    #[template(path = "docs/requirements.html")]
    struct TemplateParams<'a> {
        my_route: &'a MyRoute,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            my_route: &my_route,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn tools(my_route: MyRoute) -> HandlerResult {
    #[derive(Template)]
    #[template(path = "tools/index.html")]
    struct TemplateParams<'a> {
        my_route: &'a MyRoute,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            my_route: &my_route,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn api_v1(my_route: MyRoute) -> HandlerResult {
    #[derive(Template)]
    #[template(path = "api.html")]
    struct TemplateParams<'a> {
        my_route: &'a MyRoute,
        per_page: usize,
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            my_route: &my_route,
            per_page: crate::constants::PROJECTS_PER_PAGE,
        }
        .render()?,
    )
        .into_response())
}
