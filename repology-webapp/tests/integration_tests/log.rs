// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "log_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/log/10").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "log_data"))]
async fn test_invalid_id(pool: PgPool) {
    let response = Request::new(pool, "/log/foo").perform().await;
    assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "log_data"))]
async fn test_ongoing(pool: PgPool) {
    let response = Request::new(pool, "/log/1").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("ongoing"));
    assert!(!response.text().unwrap().contains("successful"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "log_data"))]
async fn test_finished(pool: PgPool) {
    let response = Request::new(pool, "/log/2").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("successful"));
    assert!(!response.text().unwrap().contains("ongoing"));
}
