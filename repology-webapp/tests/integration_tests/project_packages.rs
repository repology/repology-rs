// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/project/nonexistent/packages").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Unknown project"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_orphaned(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned/packages").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Gone project"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_normal(pool: PgPool) {
    let response = Request::new(pool, "/project/zsh/packages").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Packages for <strong>zsh"));
    assert!(response.text().unwrap().contains("FreeBSD"));
    assert!(response.text().unwrap().contains("1.1"));
}
