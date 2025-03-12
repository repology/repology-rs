// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use insta::assert_snapshot;
use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_empty(pool: PgPool) {
    let response = Request::new(pool, "/graph/map_repo_size_fresh.svg").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_map_data.sql"))]
async fn test_normal(pool: PgPool) {
    // XXX: forcing different different url
    let response = Request::new(pool, "/graph/map_repo_size_fresh.svg?").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_map_data.sql"))]
async fn test_limited(pool: PgPool) {
    let response = Request::new(pool, "/graph/map_repo_size_fresh.svg?xlimit=10000&ylimit=10000").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_map_data.sql"))]
async fn test_over_limited(pool: PgPool) {
    let response = Request::new(pool, "/graph/map_repo_size_fresh.svg?xlimit=1&ylimit=1").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_map_data.sql"))]
async fn test_zero_limited(pool: PgPool) {
    let response = Request::new(pool, "/graph/map_repo_size_fresh.svg?xlimit=0&ylimit=0").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
