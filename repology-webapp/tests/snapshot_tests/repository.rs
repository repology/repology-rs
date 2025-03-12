// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use insta::assert_snapshot;
use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/repository/nonexistent").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_orphaned(pool: PgPool) {
    let response = Request::new(pool, "/repository/orphaned").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_empty(pool: PgPool) {
    let response = Request::new(pool, "/repository/empty").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_stripped(pool: PgPool) {
    let response = Request::new(pool, "/repository/stripped").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_normal(pool: PgPool) {
    let response = Request::new(pool, "/repository/good").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
