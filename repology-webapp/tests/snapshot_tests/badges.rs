// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_badge_version_for_repo(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/badge/version-for-repo/freebsd/zsh").await;
    uri_snapshot_test(pool.clone(), "/badge/version-for-repo/freebsd/zsh.svg").await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/version-for-repo/freebsd/zsh.svg?minversion=1.2",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/version-for-repo/freebsd/zsh.svg?header=fbsd+ver",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/version-for-repo/freebsd/zsh.svg?header=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/version-for-repo/freebsd/unpackaged.svg",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/badge/version-for-repo/ubuntu_24/zsh.svg").await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/version-for-repo/ubuntu_24/zsh.svg?allow_ignored=1",
    )
    .await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_badge_vertical_allrepos(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/badge/vertical-allrepos/zsh").await;
    uri_snapshot_test(pool.clone(), "/badge/vertical-allrepos/unpackaged.svg").await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/vertical-allrepos/unpackaged.svg?header=Packages",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/vertical-allrepos/unpackaged.svg?header=",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/badge/vertical-allrepos/zsh.svg").await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/vertical-allrepos/zsh.svg?header=Packages",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/badge/vertical-allrepos/zsh.svg?header=").await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/vertical-allrepos/zsh.svg?allow_ignored=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/vertical-allrepos/zsh.svg?minversion=1.0",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/vertical-allrepos/zsh.svg?exclude_unsupported=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/vertical-allrepos/zsh.svg?exclude_sources=site",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository,site",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/badge/vertical-allrepos/zsh.svg?columns=4").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("badge_versions_data")
)]
async fn test_badge_latest_versions(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/badge/latest-versions/nonexistent").await;
    uri_snapshot_test(pool.clone(), "/badge/latest-versions/nonexistent.svg").await;
    uri_snapshot_test(pool.clone(), "/badge/latest-versions/zsh.svg").await;
    uri_snapshot_test(pool.clone(), "/badge/latest-versions/bash.svg").await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/latest-versions/zsh.svg?header=VERSIONS",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/badge/latest-versions/zsh.svg?header=").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "badge_repository_big_data")
)]
async fn test_badge_repositry_big(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/badge/repository-big/nonexistent").await;
    uri_snapshot_test(pool.clone(), "/badge/repository-big/nonexistent.svg").await;
    uri_snapshot_test(pool.clone(), "/badge/repository-big/ubuntu_10.svg").await;
    uri_snapshot_test(pool.clone(), "/badge/repository-big/freshcode.svg").await;
    uri_snapshot_test(pool.clone(), "/badge/repository-big/freebsd.svg").await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/repository-big/freebsd.svg?header=FreeBSD",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/badge/repository-big/freebsd.svg?header=").await;
}
