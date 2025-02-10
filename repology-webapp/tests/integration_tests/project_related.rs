// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/project/nonexistent/related").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Unknown project"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_orphaned(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned/related").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Gone project"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_no_relations(pool: PgPool) {
    let response = Request::new(pool, "/project/zsh/related").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(!response.text().unwrap().contains("gcc"));
    assert!(!response.text().unwrap().contains("binutils"));
    assert!(!response.text().unwrap().contains("∗"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_has_relations_a(pool: PgPool) {
    let response = Request::new(pool, "/project/gcc/related").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("binutils"));
    assert!(response.text().unwrap().contains("/project/binutils/versions"));
    assert!(response.text().unwrap().contains("/project/binutils/related"));
    assert!(response.text().unwrap().contains("∗"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_has_relations_b(pool: PgPool) {
    let response = Request::new(pool, "/project/binutils/related").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("gcc"));
    assert!(response.text().unwrap().contains("/project/gcc/versions"));
    assert!(response.text().unwrap().contains("/project/gcc/related"));
    assert!(response.text().unwrap().contains("∗"));
}
