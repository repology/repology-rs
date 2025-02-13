// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_missing_extension(pool: PgPool) {
    let response = Request::new(pool, "/badge/tiny-repos/nonexistent").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/badge/tiny-repos/nonexistent.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[1]/svg:text)").unwrap(), 4_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "in repositories");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[3])").unwrap(), "0");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_base(pool: PgPool) {
    let response = Request::new(pool, "/badge/tiny-repos/zsh.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[1]/svg:text)").unwrap(), 4_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "in repositories");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[3])").unwrap(), "3");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_header_custom(pool: PgPool) {
    let response = Request::new(pool, "/badge/tiny-repos/zsh.svg?header=Repository+Count")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "Repository Count");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[3])").unwrap(), "3");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_header_empty(pool: PgPool) {
    let response = Request::new(pool, "/badge/tiny-repos/zsh.svg?header=").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[1]/svg:text)").unwrap(), 2_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "3");
}
