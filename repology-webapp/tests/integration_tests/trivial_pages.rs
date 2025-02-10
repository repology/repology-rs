// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_api(pool: PgPool) {
    let response = Request::new(pool, "/api").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Terms of use"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_api_v1(pool: PgPool) {
    let response = Request::new(pool, "/api/v1").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Terms of use"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_about(pool: PgPool) {
    let response = Request::new(pool, "/docs/about").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("About"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_bots(pool: PgPool) {
    let response = Request::new(pool, "/docs/bots").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("+https://repology.org/docs/bots"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs(pool: PgPool) {
    let response = Request::new(pool, "/docs").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Documentation"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_not_supported(pool: PgPool) {
    let response = Request::new(pool, "/docs/not_supported").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Hyperbola"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_requirements(pool: PgPool) {
    let response = Request::new(pool, "/docs/requirements").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Rational"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_news(pool: PgPool) {
    let response = Request::new(pool, "/news").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Added"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_tools(pool: PgPool) {
    let response = Request::new(pool, "/tools").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Project by package name"));
}
