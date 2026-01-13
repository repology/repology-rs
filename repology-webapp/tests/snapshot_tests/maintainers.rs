// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_base(pool: PgPool) {
    let response = Request::new(pool, "/maintainers/").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_pagination_from(pool: PgPool) {
    let response = Request::new(pool, "/maintainers/aaa/").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_pagination_to(pool: PgPool) {
    let response = Request::new(pool, "/maintainers/..zzz/").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search(pool: PgPool) {
    let response = Request::new(pool, "/maintainers/?search=ctiv").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search_nomatch(pool: PgPool) {
    let response = Request::new(pool, "/maintainers/?search=nonononono").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}
