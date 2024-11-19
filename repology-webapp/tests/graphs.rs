// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_graphs(pool: PgPool) {
    check_response!(
        pool,
        "/graph/total/packages.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/total/packages.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/total/packages.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
    check_response!(
        pool,
        "/graph/total/packages.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:line[1])" 1_f64,
    );
}
