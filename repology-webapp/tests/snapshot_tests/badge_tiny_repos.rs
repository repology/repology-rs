// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_nonexistent(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/badge/tiny-repos/nonexistent").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_nonexistent_svg(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/badge/tiny-repos/nonexistent.svg").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_normal(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/badge/tiny-repos/zsh.svg").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_header_flag(pool: PgPool) {
    uri_snapshot_test(
        pool.clone(),
        "/badge/tiny-repos/zsh.svg?header=Repository+Count",
    )
    .await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_header_flag_empty(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/badge/tiny-repos/zsh.svg?header=").await;
}
