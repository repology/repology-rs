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

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_history_data")
)]
async fn test_project_history(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/nonexistent/hitstory").await;
    uri_snapshot_test(pool.clone(), "/project/orphaned-with-history/history").await;
    uri_snapshot_test(pool.clone(), "/project/orphaned-without-history/history").await;
    uri_snapshot_test(pool.clone(), "/project/zsh/history").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_project_cves(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/cves/history").await;
    uri_snapshot_test(pool.clone(), "/project/orphaned-without-cves/cves").await;
    uri_snapshot_test(pool.clone(), "/project/orphaned-with-cves/cves").await;
    uri_snapshot_test(pool.clone(), "/project/manyranges/cves").await;
    uri_snapshot_test(pool.clone(), "/project/tworanges/cves").await;
    uri_snapshot_test(pool.clone(), "/project/tworanges/cves?version=1.3").await;
    uri_snapshot_test(pool.clone(), "/project/tworanges/cves?version=1.4").await;
}
