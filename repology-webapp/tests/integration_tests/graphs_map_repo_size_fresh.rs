// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_empty(pool: PgPool) {
    let response = Request::new(pool, "/graph/map_repo_size_fresh.svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_map_data.sql"))]
async fn test_normal(pool: PgPool) {
    let response = Request::new(pool, "/graph/map_repo_size_fresh.svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert!(response.text().unwrap().contains("Ubuntu 12"));
    assert!(response.text().unwrap().contains("#aabbcc"));
    assert!(response.text().unwrap().contains("Ubuntu 20"));
    assert!(response.text().unwrap().contains("#bbccdd"));
    assert!(response.text().unwrap().contains("20000"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_map_data.sql"))]
async fn test_limited(pool: PgPool) {
    let response = Request::new(pool, "/graph/map_repo_size_fresh.svg?xlimit=10000&ylimit=10000").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert!(response.text().unwrap().contains("Ubuntu 12"));
    assert!(response.text().unwrap().contains("#aabbcc"));
    assert!(!response.text().unwrap().contains("Ubuntu 20"));
    assert!(!response.text().unwrap().contains("#bbccdd"));
    assert!(response.text().unwrap().contains("10000"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_map_data.sql"))]
async fn test_over_limited(pool: PgPool) {
    let response = Request::new(pool, "/graph/map_repo_size_fresh.svg?xlimit=1&ylimit=1").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert!(!response.text().unwrap().contains("Ubuntu 12"));
    assert!(!response.text().unwrap().contains("#aabbcc"));
    assert!(!response.text().unwrap().contains("Ubuntu 20"));
    assert!(!response.text().unwrap().contains("#bbccdd"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_map_data.sql"))]
async fn test_zero_limited(pool: PgPool) {
    let response = Request::new(pool, "/graph/map_repo_size_fresh.svg?xlimit=0&ylimit=0").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert!(response.text().unwrap().contains("Ubuntu 12"));
    assert!(response.text().unwrap().contains("#aabbcc"));
    assert!(response.text().unwrap().contains("Ubuntu 20"));
    assert!(response.text().unwrap().contains("#bbccdd"));
    assert!(response.text().unwrap().contains("20000"));
}
