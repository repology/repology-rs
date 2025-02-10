// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_nonexistent(pool: PgPool) {
    check_response!(pool, "/badge/tiny-repos/nonexistent", status NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_nonexistent_svg(pool: PgPool) {
    check_response!(
        pool,
        "/badge/tiny-repos/nonexistent.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[1]/svg:text)" 4_f64,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "in repositories",
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[3])" "0",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_normal(pool: PgPool) {
    check_response!(
        pool,
        "/badge/tiny-repos/zsh.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[1]/svg:text)" 4_f64,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "in repositories",
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[3])" "3",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_header_flag(pool: PgPool) {
    check_response!(
        pool,
        "/badge/tiny-repos/zsh.svg?header=Repository+Count",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "Repository Count",
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[3])" "3",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_header_flag_empty(pool: PgPool) {
    check_response!(
        pool,
        "/badge/tiny-repos/zsh.svg?header=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[1]/svg:text)" 2_f64,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "3",
    );
}
