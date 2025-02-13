// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_missing_extension(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/version-for-repo/freebsd/zsh").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_base(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/version-for-repo/freebsd/zsh.svg").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_minversion(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/version-for-repo/freebsd/zsh.svg?minversion=1.2").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_header_custom(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/version-for-repo/freebsd/zsh.svg?header=fbsd+ver").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_header_empty(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/version-for-repo/freebsd/zsh.svg?header=").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_unpackaged(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/version-for-repo/freebsd/unpackaged.svg").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_allow_ignored_base(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/version-for-repo/ubuntu_24/zsh.svg").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_allow_ignored_enabled(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/version-for-repo/ubuntu_24/zsh.svg?allow_ignored=1").await;
}
