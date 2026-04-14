// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::Template;
use axum::extract::Query;
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::result::HandlerResult;
use crate::routes::MyRoute;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub autorefresh: bool,
}

#[derive(Template)]
#[template(path = "repositories/graphs.html")]
struct TemplateParams<'a> {
    my_route: &'a MyRoute,
    autorefresh: bool,
}

#[cfg_attr(
    not(feature = "coverage"),
    tracing::instrument(skip_all, fields(query = ?query))
)]
pub async fn repositories_graphs(
    my_route: MyRoute,
    Query(query): Query<QueryParams>,
) -> HandlerResult {
    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            my_route: &my_route,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
