// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod api;
mod badge_tiny_repos;
mod badges;
mod feeds;
mod graphs; // XXX: may produce false positives due to moving timestamps
mod legacy_redirects;
mod log;
mod maintainer;
mod opensearch;
mod problems;
mod project;
mod projects;
mod repository;
mod security;
mod tool_project_by;
mod trivial_pages;

use axum::http::Request;
use sqlx::PgPool;
use tower_service::Service;

use repology_webapp::create_app;

async fn uri_snapshot_test(pool: PgPool, uri: &str) {
    let request = Request::builder()
        .uri(uri)
        .method("GET")
        .body("".to_owned())
        .expect("cannot create request");
    let mut app = create_app(pool).await.expect("create_app failed");
    let response = app.call(request).await.expect("all.call failed");

    let mut snapshot = format!("Status: {}\n", response.status().as_u16());
    for (k, v) in response.headers() {
        snapshot += &format!("Header: {}: {}\n", k, v.to_str().unwrap());
    }
    snapshot += "---\n";
    snapshot += std::str::from_utf8(
        &axum::body::to_bytes(response.into_body(), 1000000)
            .await
            .expect("getting response body failed"),
    )
    .expect("body is not utf-8");

    insta::assert_snapshot!(uri, snapshot);
}
