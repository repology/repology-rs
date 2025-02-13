// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_problems_nonexistent_repository(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/unknown/problems.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_problems_legacy_repository(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/ubuntu_10/problems.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_problems(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/problems.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_maintainers(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/maintainers.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_total(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_total.svg").with_xml_namespace("svg", "http://www.w3.org/2000/svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_unique(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_unique.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_unique_percent(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_unique_percent.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_newest(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_newest.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_newest_percent(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_newest_percent.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_outdated(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_outdated.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_outdated_percent(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_outdated_percent.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_problematic(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_problematic.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_problematic_percent(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_problematic_percent.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_vulnerable(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_vulnerable.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_vulnerable_percent(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_vulnerable_percent.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_problem_per_projects(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/problems_per_1000_projects.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_per_maintainer(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_per_maintainer.svg")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some(mime::IMAGE_SVG.as_ref()));
    assert_eq!(response.xpath("count(//svg:g[1]/svg:line[1])").unwrap(), 1_f64);
}
