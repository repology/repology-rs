// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_nonexistent(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/nonexistent/cves").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_orphaned_without_cves(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/orphaned-without-cves/cves").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_orphaned_with_cves(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/orphaned-with-cves/cves").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_ranges(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/manyranges/cves").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_version_not_highlighted(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/tworanges/cves").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_open_range_version_not_highlighted(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/tworanges/cves?version=1.3").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_version_highlighted(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/project/tworanges/cves?version=1.4").await;
}
