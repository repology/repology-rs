// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_nonexistent(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/api/v1/project/nonexistent").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_full(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/api/v1/project/full").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_minimal(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/api/v1/project/minimal").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_vulnerable(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/api/v1/project/vulnerable").await;
}
