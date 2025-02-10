// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/project/nonexistent/cves").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_orphaned_without_cves(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned-without-cves/cves").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_orphaned_with_cves(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned-with-cves/cves").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("CVE-1-1"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_ranges(pool: PgPool) {
    let response = Request::new(pool, "/project/manyranges/cves").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("CVE-2-2"));
    assert!(response.text().unwrap().contains("(-∞, +∞)"));
    assert!(response.text().unwrap().contains("(1.1, +∞)"));
    assert!(response.text().unwrap().contains("[1.2, +∞)"));
    assert!(response.text().unwrap().contains("(1.3, 1.4]"));
    assert!(response.text().unwrap().contains("[1.5, 1.6)"));
    assert!(response.text().unwrap().contains("(-∞, 1.7)"));
    assert!(response.text().unwrap().contains("(-∞, 1.8]"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_version_not_highlighted(pool: PgPool) {
    let response = Request::new(pool, "/project/tworanges/cves").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("CVE-3-3"));
    assert!(response.text().unwrap().contains("(1.3, 1.4]"));
    assert!(response.text().unwrap().contains("[1.5, 1.6)"));
    // css class used for highlighted entries, here we don't expect any
    assert!(!response.text().unwrap().contains("version-outdated"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_open_range_version_not_highlighted(pool: PgPool) {
    let response = Request::new(pool, "/project/tworanges/cves?version=1.3").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains(r#"<span class="version version-rolling">(1.3, 1.4]</span>"#));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_version_highlighted(pool: PgPool) {
    let response = Request::new(pool, "/project/tworanges/cves?version=1.4").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains(r#"<span class="version version-outdated">(1.3, 1.4]</span>"#));
}
