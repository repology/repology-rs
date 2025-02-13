// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_nonexistent(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainer/nonexistent@example.com").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_orphaned(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainer/orphaned@example.com").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_orphaned_in_future(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainer/orphaned-in-future@example.com").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_active(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainer/active@example.com").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_fallback(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainer/fallback-mnt-foo@repology").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_no_vuln_column(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainer/no-vuln-column@example.com").await;
}
