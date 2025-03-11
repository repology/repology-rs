// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_nonexistent(pool: PgPool) {
    uri_snapshot_test(pool, "/project/nonexistent/versions-compact").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_orphaned(pool: PgPool) {
    uri_snapshot_test(pool, "/project/orphaned/versions-compact").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_normal(pool: PgPool) {
    uri_snapshot_test(pool, "/project/zsh/versions-compact").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_xxx_attempt(pool: PgPool) {
    uri_snapshot_test(pool, "/project/%3Cmarquee%3E%26nbsp%3B/versions-compact").await;
}
