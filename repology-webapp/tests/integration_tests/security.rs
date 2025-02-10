// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("project_cves_data"))]
async fn test_recent_cves(pool: PgPool) {
    let response = Request::new(pool, "/security/recent-cves").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    // we cannot really check presence of cve here because it depends on current date
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("project_cves_data"))]
async fn test_recent_cpes(pool: PgPool) {
    let response = Request::new(pool, "/security/recent-cpes").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("orphaned-with-cves"));
    assert!(response.text().unwrap().contains("foo"));
    assert!(response.text().unwrap().contains("manyranges"));
    assert!(response.text().unwrap().contains("bar"));
    assert!(response.text().unwrap().contains("tworanges"));
    assert!(response.text().unwrap().contains("baz"));
}
