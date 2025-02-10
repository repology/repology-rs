// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_repository_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/repository/nonexistent/problems").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_repository_normal(pool: PgPool) {
    let response = Request::new(pool, "/repository/freebsd/problems").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));

    // contains maintainer column
    assert!(response.text().unwrap().contains(r#"<td class="text-center"><a href="/maintainer/johndoe"#));

    // each error kind is present
    assert!(response.text().unwrap().contains("Homepage link <code>https://example.com/</code> is <"));
    assert!(response.text().unwrap().contains("Homepage link <code>https://example.com/</code> is a permanent redirect"));
    assert!(response.text().unwrap().contains("points to Google Code which was discontinued"));
    assert!(response.text().unwrap().contains("points to codeplex which was discontinued"));
    assert!(response.text().unwrap().contains("points to Gna which was discontinued"));
    assert!(response.text().unwrap().contains("points to CPAN which was discontinued"));
    assert!(response.text().unwrap().contains("was not found neither among known CVEs nor in NVD CPE dictionary"));
    assert!(response.text().unwrap().contains("CPE information is missing for this package"));
    assert!(response.text().unwrap().contains("Download link <code>https://example.com/</code> is <"));
    assert!(response.text().unwrap().contains("Download link <code>https://example.com/</code> is a permanent redirect"));
    assert!(response.text().unwrap().contains("needs a trailing slash added"));

    assert!(!response.text().unwrap().contains("Unformatted problem of type"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_maintainer_nonexistent_repository(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/johndoe@example.com/problems-for-repo/nonexistent").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_maintainer_normal(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/johndoe@example.com/problems-for-repo/freebsd").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));

    // does not contain maintainer column, because maintainer is fixed
    assert!(!response.text().unwrap().contains(r#"<td class="text-center"><a href="/maintainer/johndoe"#));

    // each error kind is present
    assert!(response.text().unwrap().contains("Homepage link <code>https://example.com/</code> is <"));
    assert!(response.text().unwrap().contains("Homepage link <code>https://example.com/</code> is a permanent redirect"));
    assert!(response.text().unwrap().contains("points to Google Code which was discontinued"));
    assert!(response.text().unwrap().contains("points to codeplex which was discontinued"));
    assert!(response.text().unwrap().contains("points to Gna which was discontinued"));
    assert!(response.text().unwrap().contains("points to CPAN which was discontinued"));
    assert!(response.text().unwrap().contains("was not found neither among known CVEs nor in NVD CPE dictionary"));
    assert!(response.text().unwrap().contains("CPE information is missing for this package"));
    assert!(response.text().unwrap().contains("Download link <code>https://example.com/</code> is <"));
    assert!(response.text().unwrap().contains("Download link <code>https://example.com/</code> is a permanent redirect"));
    assert!(response.text().unwrap().contains("needs a trailing slash added"));

    assert!(!response.text().unwrap().contains("Unformatted problem of type"));
}
