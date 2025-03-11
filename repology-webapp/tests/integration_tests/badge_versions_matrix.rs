// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_base(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=zsh,fish")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert!(response.text().unwrap().contains("FreeBSD"));
    assert!(response.text().unwrap().contains("Ubuntu 12"));
    assert!(response.text().unwrap().contains("Ubuntu 24"));
    assert!(response.text().unwrap().contains("freshcode.club"));
    assert!(!response.text().unwrap().contains("#e00000")); // color denoting unsuitable version
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_require_all(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=zsh,fish&require_all=1")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert!(response.text().unwrap().contains("FreeBSD"));
    assert!(!response.text().unwrap().contains("Ubuntu 24"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_little_repos(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=fish")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert!(response.text().unwrap().contains("FreeBSD"));
    assert!(!response.text().unwrap().contains("Ubuntu 24"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_force_missing_repo(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=fish&repos=freebsd,ubuntu_24")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert!(response.text().unwrap().contains("FreeBSD"));
    assert!(response.text().unwrap().contains("Ubuntu 24"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_header(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=zsh,fish&header=Custom%20header")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert!(response.text().unwrap().contains("Custom header"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_exclude_unsupported(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=zsh,fish&exclude_unsupported=1")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert!(response.text().unwrap().contains("FreeBSD"));
    assert!(!response.text().unwrap().contains("Ubuntu 12"));
    assert!(response.text().unwrap().contains("Ubuntu 24"));
    assert!(response.text().unwrap().contains("freshcode.club"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_exclude_sources(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=zsh,fish&exclude_sources=site")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert!(response.text().unwrap().contains("FreeBSD"));
    assert!(response.text().unwrap().contains("Ubuntu 12"));
    assert!(response.text().unwrap().contains("Ubuntu 24"));
    assert!(!response.text().unwrap().contains("freshcode.club"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_limit_version(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=zsh%3C1.1,fish")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert!(response.text().unwrap().contains("#e00000")); // color denoting unsuitable version
}
