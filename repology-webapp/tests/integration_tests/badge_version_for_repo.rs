// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_missing_extension(pool: PgPool) {
    let response = Request::new(pool, "/badge/version-for-repo/freebsd/zsh").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/badge/version-for-repo/badrepo/zsh.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_base(pool: PgPool) {
    let response = Request::new(pool, "/badge/version-for-repo/freebsd/zsh.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[1]/svg:text)").unwrap(), 4_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "FreeBSD port");
    assert_eq!(response.xpath("count(//svg:g[1]/svg:rect[@fill='#4c1'])").unwrap(), 1_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[3])").unwrap(), "1.1");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_minversion(pool: PgPool) {
    let response = Request::new(pool, "/badge/version-for-repo/freebsd/zsh.svg?minversion=1.2")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "FreeBSD port");
    assert_eq!(response.xpath("count(//svg:g[1]/svg:rect[@fill='#e00000'])").unwrap(), 1_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[3])").unwrap(), "1.1");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_header_custom(pool: PgPool) {
    let response = Request::new(pool, "/badge/version-for-repo/freebsd/zsh.svg?header=fbsd+ver")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "fbsd ver");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[3])").unwrap(), "1.1");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_header_empty(pool: PgPool) {
    let response = Request::new(pool, "/badge/version-for-repo/freebsd/zsh.svg?header=")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:g[1]/svg:text)").unwrap(), 2_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "1.1");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_unpackaged(pool: PgPool) {
    let response = Request::new(pool, "/badge/version-for-repo/freebsd/unpackaged.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "FreeBSD port");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[3])").unwrap(), "-");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_allow_ignored_base(pool: PgPool) {
    let response = Request::new(pool, "/badge/version-for-repo/ubuntu_24/zsh.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "Ubuntu 24 package");
    assert_eq!(response.xpath("count(//svg:g[1]/svg:rect[@fill='#e05d44'])").unwrap(), 1_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[3])").unwrap(), "1.0");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_allow_ignored_enabled(pool: PgPool) {
    let response = Request::new(pool, "/badge/version-for-repo/ubuntu_24/zsh.svg?allow_ignored=1")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[1])").unwrap(), "Ubuntu 24 package");
    assert_eq!(response.xpath("count(//svg:g[1]/svg:rect[@fill='#9f9f9f'])").unwrap(), 1_f64);
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[1]/svg:text[3])").unwrap(), "1.2");
}
