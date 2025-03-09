// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_base(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainers/").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_pagination_from(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainers/aaa/").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_pagination_to(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainers/..zzz/").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainers/?search=ctiv").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search_nomatch(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainers/?search=nonononono").await;
}
