// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::response::{IntoResponse, Response};

pub async fn handle_htmx() -> Response {
    (
        [("content-type", "text/javascript")],
        include_str!("../../static/htmx.min.js"),
    )
        .into_response()
}

pub async fn handle_bulma() -> Response {
    (
        [("content-type", "text/css")],
        include_str!("../../static/bulma.min.css"),
    )
        .into_response()
}

pub async fn handle_css() -> Response {
    (
        [("content-type", "text/css")],
        include_str!("../../static/repology-admin.css"),
    )
        .into_response()
}
