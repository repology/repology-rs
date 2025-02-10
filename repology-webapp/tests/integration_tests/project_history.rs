// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_history_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/project/nonexistent/history").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_history_data"))]
async fn test_orphaned_without_history(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned-without-history/history").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_history_data"))]
async fn test_orphaned_with_history(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned-with-history/history").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_history_data"))]
async fn test_normal(pool: PgPool) {
    let response = Request::new(pool, "/project/zsh/history").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    // rather complex templates which may lead to whitespace before comma in some lists
    assert!(!response.text().unwrap().contains(" ,"));
    assert!(!response.text().unwrap().contains("\t,"));
}
