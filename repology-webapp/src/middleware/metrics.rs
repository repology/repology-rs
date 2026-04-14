// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Instant;

use axum::body::HttpBody;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use metrics::{counter, histogram};

use crate::routes::MyRoute;

pub async fn metrics_middleware(
    route: Option<MyRoute>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let start = Instant::now();
    let response = next.run(request).await;
    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    // normalize some paths which lead to the same endpoints; XXX this will hopefully be gone
    // someday when endpoints are redesigned (e.g. /projects/{bound}/ → /projects/?start=)
    let path_for_metrics = route
        .as_ref()
        .map(|route| {
            let mut path = route
                .path()
                .trim_end_matches("{bound}/")
                .trim_end_matches("/{sorting}");
            if path.starts_with("/graph/total/") {
                path = "/graph/total/..."
            }
            if path.starts_with("/graph/repo/") {
                path = "/graph/repo/..."
            }
            path
        })
        .unwrap_or("???");

    counter!("repology_webapp_http_requests_total", "path" => path_for_metrics, "status" => status)
        .increment(1);
    histogram!("repology_webapp_http_requests_duration_seconds", "path" => path_for_metrics)
        .record(latency);

    if let Some(body_size) = response.body().size_hint().exact() {
        histogram!("repology_webapp_http_response_size_bytes", "path" => path_for_metrics)
            .record(body_size as f64);
    }

    response
}
