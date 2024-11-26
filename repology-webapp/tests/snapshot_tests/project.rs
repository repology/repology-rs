// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_project_versions(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/nonexistent/versions").await;
    uri_snapshot_test(pool.clone(), "/project/orphaned/versions").await;
    uri_snapshot_test(pool.clone(), "/project/zsh/versions").await;
    uri_snapshot_test(pool.clone(), "/project/%3Cmarquee%3E%26nbsp%3B/versions").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_project_packages(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/nonexistent/packages").await;
    uri_snapshot_test(pool.clone(), "/project/orphaned/packages").await;
    uri_snapshot_test(pool.clone(), "/project/zsh/packages").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_project_information(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/nonexistent/information").await;
    uri_snapshot_test(pool.clone(), "/project/orphaned/information").await;
    uri_snapshot_test(pool.clone(), "/project/zsh/information").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_project_badges(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/nonexistent/badges").await;
    uri_snapshot_test(pool.clone(), "/project/orphaned/badges").await;
    uri_snapshot_test(pool.clone(), "/project/zsh/badges").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "related_data")
)]
async fn test_project_related(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/nonexistent/related").await;
    uri_snapshot_test(pool.clone(), "/project/orphaned/related").await;
    uri_snapshot_test(pool.clone(), "/project/zsh/related").await;
    uri_snapshot_test(pool.clone(), "/project/gcc/related").await;
    uri_snapshot_test(pool.clone(), "/project/binutils/related").await;
}
