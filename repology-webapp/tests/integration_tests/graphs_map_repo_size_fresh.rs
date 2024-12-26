// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_empty(pool: PgPool) {
    check_response!(
        pool,
        "/graph/map_repo_size_fresh.svg",
        status OK,
        content_type IMAGE_SVG,
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("graphs_map_data.sql")
)]
async fn test_normal(pool: PgPool) {
    check_response!(
        pool,
        "/graph/map_repo_size_fresh.svg",
        status OK,
        content_type IMAGE_SVG,
        contains "Ubuntu 12",
        contains "#aabbcc",
        contains "Ubuntu 20",
        contains "#bbccdd",
        contains "20000",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("graphs_map_data.sql")
)]
async fn test_limited(pool: PgPool) {
    check_response!(
        pool,
        "/graph/map_repo_size_fresh.svg?xlimit=10000&ylimit=10000",
        status OK,
        content_type IMAGE_SVG,
        contains "Ubuntu 12",
        contains "#aabbcc",
        contains_not "Ubuntu 20",
        contains_not "#bbccdd",
        contains "10000",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("graphs_map_data.sql")
)]
async fn test_over_limited(pool: PgPool) {
    check_response!(
        pool,
        "/graph/map_repo_size_fresh.svg?xlimit=1&ylimit=1",
        status OK,
        content_type IMAGE_SVG,
        contains_not "Ubuntu 12",
        contains_not "#aabbcc",
        contains_not "Ubuntu 20",
        contains_not "#bbccdd",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("graphs_map_data.sql")
)]
async fn test_zero_limited(pool: PgPool) {
    check_response!(
        pool,
        "/graph/map_repo_size_fresh.svg?xlimit=0&ylimit=0",
        status OK,
        content_type IMAGE_SVG,
        contains "Ubuntu 12",
        contains "#aabbcc",
        contains "Ubuntu 20",
        contains "#bbccdd",
        contains "20000",
    );
}
