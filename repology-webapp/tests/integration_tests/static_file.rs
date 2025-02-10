// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/static/nonexistent").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_by_hashed_name(pool: PgPool) {
    let response = Request::new(pool, "/static/repology.v1.6108dff405ea1a42.ico").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("image/x-icon"));
    assert_eq!(response.body_length(), 22382);
    assert_eq!(response.body_cityhash64(), 0x6108dff405ea1a42);
    assert!(response.header_value_str("cache-control").unwrap().unwrap().contains("public"));
    assert!(response.header_value_str("cache-control").unwrap().unwrap().contains("immutable"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_by_orig_name(pool: PgPool) {
    let response = Request::new(pool, "/static/repology.v1.ico").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("image/x-icon"));
    assert_eq!(response.body_length(), 22382);
    assert_eq!(response.body_cityhash64(), 0x6108dff405ea1a42);
    assert!(response.header_value_str("cache-control").unwrap().unwrap().contains("public"));
    assert!(!response.header_value_str("cache-control").unwrap().unwrap().contains("immutable"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_transfer_encoding_gzip(pool: PgPool) {
    let response = Request::new(pool, "/static/repology.v1.6108dff405ea1a42.ico").with_header("accept-encoding", "gzip").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("image/x-icon"));
    assert_eq!(response.body_length(), 3117);
    assert_eq!(response.body_cityhash64(), 10174067632225889947);
    assert!(response.header_value_str("cache-control").unwrap().unwrap().contains("public"));
    assert!(response.header_value_str("cache-control").unwrap().unwrap().contains("immutable"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_transfer_encoding_many(pool: PgPool) {
    // client may accept multiple encodings but we only support gzip
    let response = Request::new(pool, "/static/repology.v1.6108dff405ea1a42.ico")
        .with_header("accept-encoding", "br;q=1.0, gzip;q=0.8, *;q=0.1")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("image/x-icon"));
    assert_eq!(response.body_length(), 3117);
    assert_eq!(response.body_cityhash64(), 10174067632225889947);
    assert!(response.header_value_str("cache-control").unwrap().unwrap().contains("public"));
    assert!(response.header_value_str("cache-control").unwrap().unwrap().contains("immutable"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_mime_type_icon(pool: PgPool) {
    let response = Request::new(pool, "/static/repology.v1.ico").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("image/x-icon"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_mime_type_favicon(pool: PgPool) {
    let response = Request::new(pool, "/favicon.ico").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("image/x-icon"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_mime_type_js(pool: PgPool) {
    let response = Request::new(pool, "/static/repology.v2.js").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/javascript"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_mime_type_css(pool: PgPool) {
    let response = Request::new(pool, "/static/repology.v21.css").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/css"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_mime_type_png(pool: PgPool) {
    let response = Request::new(pool, "/static/repology40x40.v1.png").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("image/png"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_mime_type_svg(pool: PgPool) {
    let response = Request::new(pool, "/static/vulnerable.v1.svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("image/svg+xml"));
}
