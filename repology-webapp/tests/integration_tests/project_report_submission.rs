// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

fn is_false(value: &bool) -> bool {
    !value
}

#[derive(Default, serde::Serialize)]
pub struct ReportSubmission {
    #[serde(skip_serializing_if = "is_false")]
    pub need_verignore: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub need_split: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub need_merge: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub need_vuln: bool,
    pub comment: String,
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
