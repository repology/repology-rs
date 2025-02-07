// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod body;
mod headers;
mod html;
mod json;
mod snapshot;
mod status;
mod text;
mod xml;

use repology_webapp_test_utils::{Request, Response};

pub fn create_mock_router(content: &str) -> axum::Router {
    use axum::response::IntoResponse;
    let content = content.to_owned();
    axum::Router::new().route(
        "/",
        axum::routing::get(move || async {
            (
                http::StatusCode::OK,
                [(
                    http::header::CONTENT_TYPE,
                    http::header::HeaderValue::from_maybe_shared("text/plain").unwrap(),
                )],
                content,
            )
                .into_response()
        }),
    )
}

pub async fn perform_mock_request(content: &str) -> Response {
    Request::default()
        .with_uri("/")
        .perform_with(create_mock_router(content))
        .await
}
