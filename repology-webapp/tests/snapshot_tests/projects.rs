// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_pagination_from(pool: PgPool) {
    let response = Request::new(pool, "/projects/pkg_foo/").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_pagination_to(pool: PgPool) {
    let response = Request::new(pool, "/projects/..pkg_foo/").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search_a(pool: PgPool) {
    let response = Request::new(pool, "/projects/?search=bar").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search_b(pool: PgPool) {
    let response = Request::new(pool, "/projects/?search=foo").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_inrepo(pool: PgPool) {
    let response = Request::new(pool, "/projects/?inrepo=ubuntu_12").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_notinrepo(pool: PgPool) {
    let response = Request::new(pool, "/projects/?notinrepo=ubuntu_12").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_inrepo_newest(pool: PgPool) {
    let response = Request::new(pool, "/projects/?inrepo=ubuntu_12&newest=1").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_inrepo_outdated(pool: PgPool) {
    let response = Request::new(pool, "/projects/?inrepo=ubuntu_12&outdated=1").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
