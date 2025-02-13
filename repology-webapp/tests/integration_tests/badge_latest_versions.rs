// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_missing_extensions(pool: PgPool) {
    let response = Request::new(pool, "/badge/latest-versions/nonexistent").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/badge/latest-versions/nonexistent.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[1]/svg:text)").unwrap(), 4_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "latest packaged version");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[3])").unwrap(), "-");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_multiple_versions(pool: PgPool) {
    let response = Request::new(pool, "/badge/latest-versions/zsh.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[1]/svg:text)").unwrap(), 4_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "latest packaged versions");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[3])").unwrap(), "3.0, 1.0.0, 1_0_0, 1.0");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_single_version(pool: PgPool) {
    let response = Request::new(pool, "/badge/latest-versions/bash.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[1]/svg:text)").unwrap(), 4_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "latest packaged version");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[3])").unwrap(), "1.0");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_header_custom(pool: PgPool) {
    let response = Request::new(pool, "/badge/latest-versions/zsh.svg?header=VERSIONS")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "VERSIONS");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_header_empty(pool: PgPool) {
    let response = Request::new(pool, "/badge/latest-versions/zsh.svg?header=").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[1]/svg:text)").unwrap(), 2_f64);
}
