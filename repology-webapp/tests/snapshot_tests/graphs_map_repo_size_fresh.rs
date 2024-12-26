// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_empty(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/graph/map_repo_size_fresh.svg").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("graphs_map_data.sql")
)]
async fn test_normal(pool: PgPool) {
    // XXX: forcing different different url
    uri_snapshot_test(pool.clone(), "/graph/map_repo_size_fresh.svg?").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("graphs_map_data.sql")
)]
async fn test_limited(pool: PgPool) {
    uri_snapshot_test(
        pool.clone(),
        "/graph/map_repo_size_fresh.svg?xlimit=10000&ylimit=10000",
    )
    .await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("graphs_map_data.sql")
)]
async fn test_over_limited(pool: PgPool) {
    uri_snapshot_test(
        pool.clone(),
        "/graph/map_repo_size_fresh.svg?xlimit=1&ylimit=1",
    )
    .await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("graphs_map_data.sql")
)]
async fn test_zero_limited(pool: PgPool) {
    uri_snapshot_test(
        pool.clone(),
        "/graph/map_repo_size_fresh.svg?xlimit=0&ylimit=0",
    )
    .await;
}
