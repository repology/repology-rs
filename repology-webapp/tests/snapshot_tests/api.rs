// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_api_v1_project(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/api/v1/project/nonexistent").await;
    uri_snapshot_test(pool.clone(), "/api/v1/project/full").await;
    uri_snapshot_test(pool.clone(), "/api/v1/project/minimal").await;
    uri_snapshot_test(pool.clone(), "/api/v1/project/vulnerable").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "projects_data")
)]
async fn test_api_v1_projects(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/api/v1/projects/pkg_foo/").await;
    uri_snapshot_test(pool.clone(), "/api/v1/projects/?search=bar").await;
}
