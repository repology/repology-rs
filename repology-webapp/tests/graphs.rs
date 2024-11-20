// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_graphs_total(pool: PgPool) {
    check_response!(
        pool,
        "/graph/total/packages.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/total/projects.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/total/maintainers.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/total/problems.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories.sql", "graphs_data.sql")
)]
async fn test_graphs_repository(pool: PgPool) {
    check_response!(
        pool,
        "/graph/repo/unknown/problems.svg",
        status NOT_FOUND,
    );
    check_response!(
        pool,
        "/graph/repo/ubuntu_10/problems.svg",
        status NOT_FOUND,
    );

    check_response!(
        pool,
        "/graph/repo/freebsd/problems.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/maintainers.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_total.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_unique.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_unique_percent.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_newest.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_newest_percent.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_outdated.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_outdated_percent.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_problematic.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_problematic_percent.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_vulnerable.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_vulnerable_percent.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/problems_per_1000_projects.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_per_maintainer.svg?experimental_history=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );

    check_response!(
        pool,
        "/graph/repo/freebsd/problems.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/maintainers.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_total.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_unique.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_unique_percent.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_newest.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_newest_percent.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_outdated.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_outdated_percent.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_problematic.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_problematic_percent.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_vulnerable.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_vulnerable_percent.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/problems_per_1000_projects.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/repo/freebsd/projects_per_maintainer.svg?experimental_history=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
}
