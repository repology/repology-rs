// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_maintainer(pool: PgPool) {
    let response = Request::new(pool, "/opensearch/maintainer.xml").with_xml_namespace("os", "http://a9.com/-/spec/opensearch/1.1/").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/xml"));
    assert!(response.text().unwrap().contains("={searchTerms}"));
    assert_eq!(response.xpath("string(/os:OpenSearchDescription/os:ShortName)").unwrap(), "Repology maintainers");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_project(pool: PgPool) {
    let response = Request::new(pool, "/opensearch/project.xml").with_xml_namespace("os", "http://a9.com/-/spec/opensearch/1.1/").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/xml"));
    assert!(response.text().unwrap().contains("={searchTerms}"));
    assert_eq!(response.xpath("string(/os:OpenSearchDescription/os:ShortName)").unwrap(), "Repology projects");
}
