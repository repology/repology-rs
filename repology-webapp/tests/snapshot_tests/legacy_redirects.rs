// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_version_only_for_repo(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/badge/version-only-for-repo/foo/bar.svg").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_version_only_for_repo_with_title(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/badge/version-only-for-repo/foo/bar.svg?header=baz").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_project_root(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/zsh").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_metapackage(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/metapackage/zsh").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_metapackage_versions(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/metapackage/zsh/versions").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_metapackage_packages(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/metapackage/zsh/packages").await;
}
