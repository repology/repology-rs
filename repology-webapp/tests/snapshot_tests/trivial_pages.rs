// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_api(pool: PgPool) {
    uri_snapshot_test(pool, "/api").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_api_v1(pool: PgPool) {
    uri_snapshot_test(pool, "/api/v1").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_about(pool: PgPool) {
    uri_snapshot_test(pool, "/docs/about").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_bots(pool: PgPool) {
    uri_snapshot_test(pool, "/docs/bots").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs(pool: PgPool) {
    uri_snapshot_test(pool, "/docs").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_not_supported(pool: PgPool) {
    uri_snapshot_test(pool, "/docs/not_supported").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_requirements(pool: PgPool) {
    uri_snapshot_test(pool, "/docs/requirements").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_news(pool: PgPool) {
    uri_snapshot_test(pool, "/news").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_tools(pool: PgPool) {
    uri_snapshot_test(pool, "/tools").await;
}
