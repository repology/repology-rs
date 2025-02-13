// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_missing_extension(pool: PgPool) {
    let response = Request::new(pool, "/badge/vertical-allrepos/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

mod test_header {
    use super::*;

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_base(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])").unwrap(), "Packaging status");
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_custom(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?header=Packages")
            .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
            .perform()
            .await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])").unwrap(), "Packages");
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_custom_empty(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?header=")
            .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
            .perform()
            .await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
        assert_eq!(response.xpath("count(//svg:g[1]/svg:g[@font-size=15]/svg:text)").unwrap(), 0_f64);
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_no_packages(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/unpackaged.svg")
            .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
            .perform()
            .await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])").unwrap(), "No known packages");
        assert_eq!(response.xpath("count(//svg:g[1]/svg:g[@font-size=11]/svg:text)").unwrap(), 0_f64);
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_no_packages_custom(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/unpackaged.svg?header=Packages")
            .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
            .perform()
            .await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])").unwrap(), "Packages");
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_no_packages_custom_empty(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/unpackaged.svg?header=")
            .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
            .perform()
            .await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
        assert_eq!(response.xpath("count(//svg:g[1]/svg:g[@font-size=15]/svg:text)").unwrap(), 0_f64);
    }
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_version_flags(pool: PgPool) {
    let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])").unwrap(), "FreeBSD");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[3])").unwrap(), "1.1");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])").unwrap(), "freshcode.club");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[3])").unwrap(), "1.0");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])").unwrap(), "Ubuntu 12");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[3])").unwrap(), "0.9");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])").unwrap(), "Ubuntu 24");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[3])").unwrap(), "1.0");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:rect[2]/@fill)").unwrap(), "#4c1");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:rect[4]/@fill)").unwrap(), "#e05d44");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:rect[6]/@fill)").unwrap(), "#e05d44");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:rect[8]/@fill)").unwrap(), "#e05d44");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_allow_ignored(pool: PgPool) {
    let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?allow_ignored=1")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])").unwrap(), "Ubuntu 24");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[3])").unwrap(), "1.2");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:rect[8]/@fill)").unwrap(), "#9f9f9f");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_minversion(pool: PgPool) {
    let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?minversion=1.0")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:rect[2]/@fill)").unwrap(), "#4c1");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:rect[4]/@fill)").unwrap(), "#e05d44");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:rect[6]/@fill)").unwrap(), "#e00000");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:rect[8]/@fill)").unwrap(), "#e05d44");
}

mod test_repository_filters {
    use super::*;

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_base(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])").unwrap(), "FreeBSD");
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])").unwrap(), "freshcode.club");
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])").unwrap(), "Ubuntu 12");
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])").unwrap(), "Ubuntu 24");
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_exclude_unsupported(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?exclude_unsupported=1")
            .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
            .perform()
            .await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])").unwrap(), "FreeBSD");
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])").unwrap(), "freshcode.club");
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])").unwrap(), "Ubuntu 24");
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_exclude_sources_site(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?exclude_sources=site")
            .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
            .perform()
            .await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])").unwrap(), "FreeBSD");
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])").unwrap(), "Ubuntu 12");
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])").unwrap(), "Ubuntu 24");
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_exclude_sources_repository(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository")
            .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
            .perform()
            .await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
        assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])").unwrap(), "freshcode.club");
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_exclude_sources_many(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository,site")
            .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
            .perform()
            .await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
        assert_eq!(response.xpath("count(//svg:g[1]/svg:g[@font-size=11]/svg:text)").unwrap(), 0_f64);
    }
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_columns(pool: PgPool) {
    let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?columns=4")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])").unwrap(), "FreeBSD");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[3])").unwrap(), "1.1");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[5])").unwrap(), "freshcode.club");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[7])").unwrap(), "1.0");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[9])").unwrap(), "Ubuntu 12");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[11])").unwrap(), "0.9");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[13])").unwrap(), "Ubuntu 24");
    assert_eq!(response.xpath("string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[15])").unwrap(), "1.0");
}
