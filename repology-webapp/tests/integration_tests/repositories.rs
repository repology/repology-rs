// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics(pool: PgPool) {
    let response = Request::new(pool, "/repositories/statistics").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_newest(pool: PgPool) {
    let response = Request::new(pool, "/repositories/statistics/newest").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_pnewest(pool: PgPool) {
    let response = Request::new(pool, "/repositories/statistics/pnewest").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_outdated(pool: PgPool) {
    let response = Request::new(pool, "/repositories/statistics/outdated").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_poutdated(pool: PgPool) {
    let response = Request::new(pool, "/repositories/statistics/poutdated").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_total(pool: PgPool) {
    let response = Request::new(pool, "/repositories/statistics/total").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_nonunique(pool: PgPool) {
    let response = Request::new(pool, "/repositories/statistics/nonunique").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_vulnerable(pool: PgPool) {
    let response = Request::new(pool, "/repositories/statistics/vulnerable").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_pvulnerable(pool: PgPool) {
    let response = Request::new(pool, "/repositories/statistics/pvulnerable").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_packages(pool: PgPool) {
    let response = Request::new(pool, "/repositories/packages").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("12345"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_graphs(pool: PgPool) {
    // TODO: add some more fixure data, otherwise we're just testing an empty list
    let response = Request::new(pool, "/repositories/graphs").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_updates(pool: PgPool) {
    // TODO: add some more fixure data, otherwise we're just testing an empty list
    let response = Request::new(pool, "/repositories/updates").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_fields(pool: PgPool) {
    // TODO: add some more fixure data, otherwise we're just testing an empty list
    let response = Request::new(pool, "/repositories/fields").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}
