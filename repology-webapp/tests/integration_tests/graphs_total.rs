// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_packages(pool: PgPool) {
    let response = Request::new(pool, "/graph/total/packages.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_projects(pool: PgPool) {
    let response = Request::new(pool, "/graph/total/projects.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_maintainers(pool: PgPool) {
    let response = Request::new(pool, "/graph/total/maintainers.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_problems(pool: PgPool) {
    let response = Request::new(pool, "/graph/total/problems.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}
