// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_from(pool: PgPool) {
    uri_snapshot_test(pool, "/api/v1/projects/pkg_foo/").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search(pool: PgPool) {
    uri_snapshot_test(pool, "/api/v1/projects/?search=bar").await;
}
