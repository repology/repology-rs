// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/project/nonexistent").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_full(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/project/full").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_minimal(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/project/minimal").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_vulnerable(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/project/vulnerable").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
