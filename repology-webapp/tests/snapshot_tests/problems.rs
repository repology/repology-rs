// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use insta::assert_snapshot;
use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_repository_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/repository/nonexistent/problems").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_repository_normal(pool: PgPool) {
    let response = Request::new(pool, "/repository/freebsd/problems").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_maintainer_nonexistent_repository(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/johndoe@example.com/problems-for-repo/nonexistent").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_maintainer_normal(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/johndoe@example.com/problems-for-repo/freebsd").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
