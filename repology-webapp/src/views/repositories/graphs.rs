// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::Template;
use axum::extract::Query;
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::endpoints::MyEndpoint;
use crate::result::EndpointResult;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub autorefresh: bool,
}

#[derive(Template)]
#[template(path = "repositories/graphs.html")]
struct TemplateParams<'a> {
    endpoint: &'a MyEndpoint,
    autorefresh: bool,
}

#[cfg_attr(
    not(feature = "coverage"),
    tracing::instrument(skip_all, fields(query = ?query))
)]
pub async fn repositories_graphs(
    endpoint: MyEndpoint,
    Query(query): Query<QueryParams>,
) -> EndpointResult {
    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            endpoint: &endpoint,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
