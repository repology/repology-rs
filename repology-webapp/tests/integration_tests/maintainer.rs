// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/nonexistent@example.com").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Unknown maintainer"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_orphaned(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/orphaned@example.com").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Gone maintainer"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_orphaned_in_future(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/orphaned-in-future@example.com").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Gone maintainer"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_active(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/active@example.com").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    // fallback section
    assert!(!response.text().unwrap().contains("fallback maintainer"));
    // contact section
    assert!(response.text().unwrap().contains("mailto:active@example.com"));
    // repositories section
    assert!(response.text().unwrap().contains("FreeBSD"));
    // categories section
    assert!(response.text().unwrap().contains("games"));
    // not testing similar maintainers for now
    // not testing projects list for now
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_fallback(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/fallback-mnt-foo@repology").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    // fallback section
    assert!(response.text().unwrap().contains("fallback maintainer"));
    // contact section
    assert!(!response.text().unwrap().contains("mailto:active@example.com"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_no_vuln_column(pool: PgPool) {
    // Maintainer not updated for a long time, without vulnerable projects
    // counter filled.
    let response = Request::new(pool, "/maintainer/no-vuln-column@example.com").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    // enough to just be deserialized without errors
}
