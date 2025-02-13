// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_missing_extension(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/nonexistent").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_nonexistent_repository(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/nonexistent.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])").unwrap(), "Repository status");
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[@font-size=11]/svg:text)").unwrap(), 2_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])").unwrap(), "Repository not known or was removed");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_legacy_repository(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/ubuntu_10.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])").unwrap(), "Repository status");
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[@font-size=11]/svg:text)").unwrap(), 2_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])").unwrap(), "Repository not known or was removed");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_active_repository_without_packages(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/freshcode.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])").unwrap(), "Repository status");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])").unwrap(), "Projects total");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[3])").unwrap(), "0");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_active_repositry(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/freebsd.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])").unwrap(), "Repository status");
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[@font-size=11]/svg:text)").unwrap(), 32_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])").unwrap(), "Projects total");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[3])").unwrap(), "10");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])").unwrap(), "Up to date");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[3])").unwrap(), "1");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[5])").unwrap(), "20.00%");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])").unwrap(), "Outdated");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[3])").unwrap(), "2");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[5])").unwrap(), "40.00%");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])").unwrap(), "Vulnerable");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[3])").unwrap(), "3");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[5])").unwrap(), "30.00%");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][5]/svg:text[1])").unwrap(), "Bad versions");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][5]/svg:text[3])").unwrap(), "4");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][5]/svg:text[5])").unwrap(), "40.00%");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][6]/svg:text[1])").unwrap(), "Maintainers");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][6]/svg:text[3])").unwrap(), "7");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_header_custom(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/freebsd.svg?header=FreeBSD")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[@font-size=15])").unwrap(), 1_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])").unwrap(), "FreeBSD");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_header_empty(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/freebsd.svg?header=")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[@font-size=15])").unwrap(), 0_f64);
}
