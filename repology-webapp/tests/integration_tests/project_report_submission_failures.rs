// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

use super::project_report_submission::ReportSubmission;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_empty_form(pool: PgPool) {
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
async fn test_comment_too_long(pool: PgPool) {
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
async fn test_no_vuln(pool: PgPool) {
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
async fn test_orphaned_project(pool: PgPool) {
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
async fn test_too_many_reports(pool: PgPool) {
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
async fn test_html(pool: PgPool) {
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
async fn test_meaningless_report(pool: PgPool) {
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

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_spam_keyword(pool: PgPool) {
    let form = ReportSubmission {
        comment: "buy foobaria".to_owned(),
        need_verignore: true,
        need_split: true,
        need_merge: true,
        need_vuln: true,
    };
    let response = Request::new(pool, "/project/zsh/report").with_form(form).with_spam_keyword("foobar").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Could not add report"));
    assert!(response.text().unwrap().contains("spammers not welcome"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_disabled_project(pool: PgPool) {
    let form = ReportSubmission {
        comment: "".to_owned(),
        need_verignore: true,
        ..Default::default()
    };
    let response = Request::new(pool, "/project/zsh/report").with_form(form).with_reports_disabled("zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Could not add report"));
    assert!(response.text().unwrap().contains("new reports to this project are disabled"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_spam_network(pool: PgPool) {
    let form = ReportSubmission {
        comment: "".to_owned(),
        need_verignore: true,
        ..Default::default()
    };
    let response = Request::new(pool, "/project/zsh/report")
        .with_form(form)
        .with_spam_network(&"10.0.0.1/32".parse().unwrap())
        .with_header("x-real-ip", "10.0.0.0, 10.0.0.1, 10.0.0.2")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("Could not add report"));
    assert!(response.text().unwrap().contains("spammers not welcome"));
}
