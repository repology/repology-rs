// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::Template;
use axum::extract::{Path, Query};
use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;

use crate::endpoints::Endpoint;
use crate::result::EndpointResult;
use crate::template_context::TemplateContext;

#[derive(Template)]
#[template(path = "news.html")]
struct TemplateParams {
    ctx: TemplateContext,
}

#[tracing::instrument()]
pub async fn news(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
) -> EndpointResult {
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
