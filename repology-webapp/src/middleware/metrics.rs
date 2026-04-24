// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;
use std::time::Instant;

use axum::body::HttpBody;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use metrics::{counter, histogram};

use crate::routes::MyRoute;

fn remove_char(s: &str, ch: char) -> Cow<'_, str> {
    if s.contains('*') {
        Cow::Owned(s.chars().filter(|&c| c != ch).collect())
    } else {
        Cow::Borrowed(s)
    }
}

fn normalize_path_for_metrics(path: &str) -> Cow<'_, str> {
    // - Grafana doesn't handle asterisk in variables properly, remove it
    //   See https://github.com/grafana/grafana/issues/123517
    // - Collapse graph types, these are basically the same code
    // - Collapse optional paginated and sorted variants, these
    //   are the same as corresponding routes without pagination
    //   and sorting
    remove_char(
        if path.starts_with("/graph/total/") {
            "/graph/total/..."
        } else if path.starts_with("/graph/repo/") {
            "/graph/repo/..."
        } else {
            path.trim_end_matches("{bound}/")
                .trim_end_matches("/{sorting}")
        },
        '*',
    )
}

pub async fn metrics_middleware(
    route: Option<MyRoute>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let start = Instant::now();
    let response = next.run(request).await;
    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let path_for_metrics =
        normalize_path_for_metrics(route.as_ref().map(|route| route.path()).unwrap_or("???"));

    counter!("repology_webapp_http_requests_total", "path" => path_for_metrics.clone(), "status" => status)
        .increment(1);
    histogram!("repology_webapp_http_requests_duration_seconds", "path" => path_for_metrics.clone())
        .record(latency);

    if let Some(body_size) = response.body().size_hint().exact() {
        histogram!("repology_webapp_http_response_size_bytes", "path" => path_for_metrics)
            .record(body_size as f64);
    }

    response
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_for_metrics() {
        assert_eq!(normalize_path_for_metrics("/foo/bar/baz"), "/foo/bar/baz");

        assert_eq!(normalize_path_for_metrics("/projects/"), "/projects/");
        assert_eq!(
            normalize_path_for_metrics("/projects/{bound}/"),
            "/projects/"
        );

        assert_eq!(
            normalize_path_for_metrics("/repositories/statistics"),
            "/repositories/statistics"
        );
        assert_eq!(
            normalize_path_for_metrics("/repositories/statistics/{sorting}"),
            "/repositories/statistics"
        );

        assert_eq!(
            normalize_path_for_metrics("/graph/total/projects.svg"),
            "/graph/total/..."
        );
        assert_eq!(
            normalize_path_for_metrics("/graph/total/maintainers.svg"),
            "/graph/total/..."
        );

        assert_eq!(normalize_path_for_metrics("/links/{*url}"), "/links/{url}");
    }
}
