// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_repository(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/repository/nonexistent").await;
    uri_snapshot_test(pool.clone(), "/repository/orphaned").await;
    uri_snapshot_test(pool.clone(), "/repository/stripped").await;
    uri_snapshot_test(pool.clone(), "/repository/good").await;
}
