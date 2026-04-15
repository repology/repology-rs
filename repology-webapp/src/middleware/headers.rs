// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::extract::Request;
use axum::http::header::{self, HeaderValue};
use axum::middleware::Next;
use axum::response::IntoResponse;

use crate::routes::MyRoute;

pub async fn headers_middleware(
    route: Option<MyRoute>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let allow_embedding = route
        .map(|route| route.props().allow_embedding)
        .unwrap_or_default();
    let mut response = next.run(request).await;

    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    if !allow_embedding {
        response
            .headers_mut()
            .insert(
                header::CONTENT_SECURITY_POLICY,
                HeaderValue::from_static("default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'")
            );
        response
            .headers_mut()
            .insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    } else {
        // relaxed headers to allow some embedding cases, see https://github.com/repology/repology-webapp/issues/175
        response.headers_mut().insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; frame-ancestors *; base-uri 'none'; form-action 'self'")
        );
    }
    // NOTE: Strict-Transport-Security must be set where HTTPS is terminated, e.g. nginx

    // Repology does not contain sensitive information, while we'd like
    // to announce visits from repology to upstreams and repositories
    response.headers_mut().insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("unsafe-url"),
    );

    response
}
