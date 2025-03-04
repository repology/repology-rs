// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::Template;
use axum::extract::{Path, Query};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::endpoints::Endpoint;
use crate::result::EndpointResult;
use crate::template_context::TemplateContext;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub autorefresh: bool,
}

#[derive(Template)]
#[template(path = "repositories/graphs.html")]
struct TemplateParams {
    ctx: TemplateContext,
    autorefresh: bool,
}

#[cfg_attr(
    not(feature = "coverage"),
    tracing::instrument(skip(gen_path, gen_query))
)]
pub async fn repositories_graphs(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
    Query(query): Query<QueryParams>,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::RepositoriesGraphs, gen_path, gen_query);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
