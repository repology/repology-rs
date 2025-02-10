// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/repository/nonexistent").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    // currently no html page for 404
    //assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    //assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    //assert!(response.text().unwrap().contains("Unknown repositry"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_orphaned(pool: PgPool) {
    let response = Request::new(pool, "/repository/orphaned").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Gone repository"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_empty(pool: PgPool) {
    let response = Request::new(pool, "/repository/empty").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(!response.text().unwrap().contains("Gone repository"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_stripped(pool: PgPool) {
    // test handling minimal data in database: empty metadata, and
    // no used_package_link_types; in prod this case is possible
    // for repositories removed long time ago
    let response = Request::new(pool, "/repository/stripped").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Stripped"));
    assert!(response.text().unwrap().contains("homepage or download links"));
    assert!(response.text().unwrap().contains("package recipes or sources"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_normal(pool: PgPool) {
    let response = Request::new(pool, "/repository/good").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Good"));
    assert!(!response.text().unwrap().contains("homepage or download links"));
    assert!(!response.text().unwrap().contains("package recipes or sources"));
    assert!(response.text().unwrap().contains("https://example.com/goodrepo"));
}
