// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_repository_nonexistent(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/repository/nonexistent/problems").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_repository_normal(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/repository/freebsd/problems").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_maintainer_nonexistent_repository(pool: PgPool) {
    uri_snapshot_test(
        pool.clone(),
        "/maintainer/johndoe@example.com/problems-for-repo/nonexistent",
    )
    .await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_maintainer_normal(pool: PgPool) {
    uri_snapshot_test(
        pool.clone(),
        "/maintainer/johndoe@example.com/problems-for-repo/freebsd",
    )
    .await;
}
