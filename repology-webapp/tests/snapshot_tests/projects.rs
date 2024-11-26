// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "projects_data")
)]
async fn test_projects_pagination(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/projects/pkg_foo/").await;
    uri_snapshot_test(pool.clone(), "/projects/..pkg_foo/").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "projects_data")
)]
async fn test_projects_search(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/projects/?search=bar").await;
    uri_snapshot_test(pool.clone(), "/projects/?search=foo").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "projects_data")
)]
async fn test_projects_inrepo(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/projects/?inrepo=ubuntu_12").await;
    uri_snapshot_test(pool.clone(), "/projects/?notinrepo=ubuntu_12").await;
    uri_snapshot_test(pool.clone(), "/projects/?inrepo=ubuntu_12&newest=1").await;
    uri_snapshot_test(pool.clone(), "/projects/?inrepo=ubuntu_12&outdated=1").await;
}
