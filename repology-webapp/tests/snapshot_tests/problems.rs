// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages", "problems_data")
)]
async fn test_problems(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/repository/nonexistent/problems").await;
    uri_snapshot_test(pool.clone(), "/repository/freebsd/problems").await;
    uri_snapshot_test(
        pool.clone(),
        "/maintainer/johndoe@example.com/problems-for-repo/nonexistent",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/maintainer/johndoe@example.com/problems-for-repo/freebsd",
    )
    .await;
}
