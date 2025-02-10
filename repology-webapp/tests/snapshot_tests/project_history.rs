// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_history_data"))]
async fn test_nonexistent(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/nonexistent/hitstory").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_history_data"))]
async fn test_orphaned_with_history(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/orphaned-with-history/history").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_history_data"))]
async fn test_orphaned_without_history(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/orphaned-without-history/history").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_history_data"))]
async fn test_normal(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/zsh/history").await;
}
