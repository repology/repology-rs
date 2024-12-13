// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/project/nonexistent/report").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(!response.text().unwrap().contains("New report"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_orphaned_without_reports(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned-without-reports/report").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(!response.text().unwrap().contains("New report"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_orphaned_with_reports(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned-with-reports/report").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(!response.text().unwrap().contains("New report"));
    assert!(response.text().unwrap().contains("Project is gone, so new reports are disabled"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_too_many_reports(pool: PgPool) {
    let response = Request::new(pool, "/project/many-reports/report").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(!response.text().unwrap().contains("New report"));
    assert!(response.text().unwrap().contains("There are too many reports for this project"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_xss_attempt(pool: PgPool) {
    let response = Request::new(pool, "/project/xss-attempt/report").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(!response.text().unwrap().contains("<a href=\"malicious\">"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_has_new_report_form(pool: PgPool) {
    let response = Request::new(pool, "/project/zsh/report").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("New report"));
}

