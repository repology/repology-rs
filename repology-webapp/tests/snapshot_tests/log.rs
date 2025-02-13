// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "log_data"))]
async fn test_nonexistent(pool: PgPool) {
    uri_snapshot_test(pool, "/log/10").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "log_data"))]
async fn test_invalid_id(pool: PgPool) {
    uri_snapshot_test(pool, "/log/foo").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "log_data"))]
async fn test_ongoing(pool: PgPool) {
    uri_snapshot_test(pool, "/log/1").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "log_data"))]
async fn test_finished(pool: PgPool) {
    uri_snapshot_test(pool, "/log/2").await;
}
