// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::Template;
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;

use crate::result::HandlerResult;

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn opensearch_project() -> HandlerResult {
    #[derive(Template)]
    #[template(path = "opensearch/project.xml")]
    struct TemplateParams;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        )],
        TemplateParams.render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn opensearch_maintainer() -> HandlerResult {
    #[derive(Template)]
    #[template(path = "opensearch/maintainer.xml")]
    struct TemplateParams;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        )],
        TemplateParams.render()?,
    )
        .into_response())
}
