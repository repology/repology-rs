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

fn is_false(value: &bool) -> bool {
    !value
}

#[derive(Default, serde::Serialize)]
struct ReportSubmission {
    #[serde(skip_serializing_if = "is_false")]
    need_verignore: bool,
    #[serde(skip_serializing_if = "is_false")]
    need_split: bool,
    #[serde(skip_serializing_if = "is_false")]
    need_merge: bool,
    #[serde(skip_serializing_if = "is_false")]
    need_vuln: bool,
    comment: String,
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_submit_report_success(pool: PgPool) {
    let form = ReportSubmission {
        comment: "My Test Report".to_owned(),
        ..Default::default()
    };
    let response = Request::new(pool.clone(), "/project/zsh/report").with_form(form).perform().await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/project/zsh/report"));

    let response = Request::new(pool, "/project/zsh/report").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("My Test Report"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_submit_report_success_vunl(pool: PgPool) {
    let form = ReportSubmission {
        comment: "nvd.nist.gov/vuln/detail/CVE-12345".to_owned(),
        need_vuln: true,
        ..Default::default()
    };
    let response = Request::new(pool.clone(), "/project/zsh/report").with_form(form).perform().await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/project/zsh/report"));

    let response = Request::new(pool, "/project/zsh/report").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("nvd.nist.gov/vuln/detail/CVE-12345"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_submit_report_empty_form(pool: PgPool) {
    let form = ReportSubmission {
        comment: "".to_owned(),
        ..Default::default()
    };
    let response = Request::new(pool, "/project/zsh/report").with_form(form).perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Could not add report"));
    assert!(response.text().unwrap().contains("please fill out the form"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_submit_report_comment_too_long(pool: PgPool) {
    let form = ReportSubmission {
        comment: std::iter::repeat('x').take(1024 * 100).collect(),
        ..Default::default()
    };
    let response = Request::new(pool, "/project/zsh/report").with_form(form).perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Could not add report"));
    assert!(response.text().unwrap().contains("comment is too long"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_submit_report_no_vuln(pool: PgPool) {
    let form = ReportSubmission {
        comment: "Vuln is missing".to_owned(),
        need_vuln: true,
        ..Default::default()
    };
    let response = Request::new(pool, "/project/zsh/report").with_form(form).perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Could not add report"));
    assert!(response.text().unwrap().contains("link to NVD entry"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_submit_report_orphaned(pool: PgPool) {
    let form = ReportSubmission {
        comment: "Vuln is missing".to_owned(),
        need_vuln: true,
        ..Default::default()
    };
    let response = Request::new(pool, "/project/orphaned-with-reports/report").with_form(form).perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Could not add report"));
    assert!(response.text().unwrap().contains("project does not exist or is gone"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_submit_report_too_many(pool: PgPool) {
    let form = ReportSubmission {
        comment: "Spam".to_owned(),
        ..Default::default()
    };
    let response = Request::new(pool, "/project/many-reports/report").with_form(form).perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Could not add report"));
    assert!(response.text().unwrap().contains("too many reports"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_submit_report_html(pool: PgPool) {
    let form = ReportSubmission {
        comment: "<a href=\"spam spam spam\">spam</a>".to_owned(),
        need_vuln: true,
        ..Default::default()
    };
    let response = Request::new(pool, "/project/zsh/report").with_form(form).perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Could not add report"));
    assert!(response.text().unwrap().contains("HTML not allowed"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_submit_report_meaningless_spam(pool: PgPool) {
    let form = ReportSubmission {
        comment: "".to_owned(),
        need_verignore: true,
        need_split: true,
        need_merge: true,
        need_vuln: true,
    };
    let response = Request::new(pool, "/project/zsh/report").with_form(form).perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Could not add report"));
    assert!(response.text().unwrap().contains("spammers not welcome"));
}
