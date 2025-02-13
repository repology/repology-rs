// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_nonexistent(pool: PgPool) {
    uri_snapshot_test(pool, "/project/nonexistent/related").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_orphaned(pool: PgPool) {
    uri_snapshot_test(pool, "/project/orphaned/related").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_no_relations(pool: PgPool) {
    uri_snapshot_test(pool, "/project/zsh/related").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_has_relations_a(pool: PgPool) {
    uri_snapshot_test(pool, "/project/gcc/related").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_has_relations_b(pool: PgPool) {
    uri_snapshot_test(pool, "/project/binutils/related").await;
}
