// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_missing_extension(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/repository-big/nonexistent").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_nonexistent_repository(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/repository-big/nonexistent.svg").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_legacy_repository(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/repository-big/ubuntu_10.svg").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_active_repository_without_packages(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/repository-big/freshcode.svg").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_active_repository(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/repository-big/freebsd.svg").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_header_custom(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/repository-big/freebsd.svg?header=FreeBSD").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_header_empty(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/repository-big/freebsd.svg?header=").await;
}
