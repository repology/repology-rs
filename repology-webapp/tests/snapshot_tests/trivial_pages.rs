// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_api(pool: PgPool) {
    let response = Request::new(pool, "/api").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_api_v1(pool: PgPool) {
    let response = Request::new(pool, "/api/v1").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_about(pool: PgPool) {
    let response = Request::new(pool, "/docs/about").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_bots(pool: PgPool) {
    let response = Request::new(pool, "/docs/bots").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs(pool: PgPool) {
    let response = Request::new(pool, "/docs").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_not_supported(pool: PgPool) {
    let response = Request::new(pool, "/docs/not_supported").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_requirements(pool: PgPool) {
    let response = Request::new(pool, "/docs/requirements").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_news(pool: PgPool) {
    let response = Request::new(pool, "/news").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_tools(pool: PgPool) {
    let response = Request::new(pool, "/tools").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
