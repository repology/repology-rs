// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use tower_cookies::{Cookie, Cookies};

pub async fn honeypot_middleware(
    cookies: Cookies,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    // denotes that it's not a first visit to repology
    cookies.add(Cookie::new("a", "1"));

    // denotes that the honeypot was visited
    if request.uri().path().starts_with("/honeypot") {
        cookies.add(Cookie::new("b", "1"));
    }

    next.run(request).await
}
