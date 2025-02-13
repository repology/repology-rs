// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_pagination_from(pool: PgPool) {
    uri_snapshot_test(pool, "/projects/pkg_foo/").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_pagination_to(pool: PgPool) {
    uri_snapshot_test(pool, "/projects/..pkg_foo/").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search_a(pool: PgPool) {
    uri_snapshot_test(pool, "/projects/?search=bar").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search_b(pool: PgPool) {
    uri_snapshot_test(pool, "/projects/?search=foo").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_inrepo(pool: PgPool) {
    uri_snapshot_test(pool, "/projects/?inrepo=ubuntu_12").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_notinrepo(pool: PgPool) {
    uri_snapshot_test(pool, "/projects/?notinrepo=ubuntu_12").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_inrepo_newest(pool: PgPool) {
    uri_snapshot_test(pool, "/projects/?inrepo=ubuntu_12&newest=1").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_inrepo_outdated(pool: PgPool) {
    uri_snapshot_test(pool, "/projects/?inrepo=ubuntu_12&outdated=1").await;
}
