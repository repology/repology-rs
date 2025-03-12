// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_sitemap_index(pool: PgPool) {
    let response = Request::new(pool, "/sitemaps/index.xml").with_xml_namespace("s", "http://www.sitemaps.org/schemas/sitemap/0.9").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/xml"));
    assert_eq!(response.xpath("count(/s:sitemapindex/s:sitemap/s:loc)").unwrap(), 4_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_sitemap_main(pool: PgPool) {
    let response = Request::new(pool, "/sitemaps/main.xml").with_xml_namespace("s", "http://www.sitemaps.org/schemas/sitemap/0.9").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/xml"));
    assert_eq!(response.xpath("count(/s:urlset/s:url/s:loc)").unwrap(), 8_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_sitemap_repositores(pool: PgPool) {
    let response = Request::new(pool, "/sitemaps/repositories.xml")
        .with_xml_namespace("s", "http://www.sitemaps.org/schemas/sitemap/0.9")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/xml"));
    assert_eq!(response.xpath("count(/s:urlset/s:url/s:loc)").unwrap(), 4_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_maintainers"))]
async fn test_sitemap_maintainers(pool: PgPool) {
    let response = Request::new(pool, "/sitemaps/maintainers.xml")
        .with_xml_namespace("s", "http://www.sitemaps.org/schemas/sitemap/0.9")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/xml"));
    assert_eq!(response.xpath("count(/s:urlset/s:url/s:loc)").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("sitemap_projects"))]
async fn test_sitemap_projects(pool: PgPool) {
    let response = Request::new(pool, "/sitemaps/projects.xml").with_xml_namespace("s", "http://www.sitemaps.org/schemas/sitemap/0.9").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/xml"));
    assert_eq!(response.xpath("count(/s:urlset/s:url/s:loc)").unwrap(), 1_f64);
}
