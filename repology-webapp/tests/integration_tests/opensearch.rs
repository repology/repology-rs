// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;
use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_maintainer(pool: PgPool) {
    let response = Request::new(pool, "/opensearch/maintainer.xml")
        .with_xml_namespace("os", "http://a9.com/-/spec/opensearch/1.1/")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/xml"));
    assert!(response.text().unwrap().contains("={searchTerms}"));
    assert_eq!(response.xpath("string(/os:OpenSearchDescription/os:ShortName)").unwrap(), "Repology maintainers");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_project(pool: PgPool) {
    check_response!(
        pool,
        "/opensearch/project.xml",
        status OK,
        content_type "application/xml",
        contains "={searchTerms}",
        xpath "string(/*[local-name()='OpenSearchDescription']/*[local-name()='ShortName'])" "Repology projects"
    );
}
