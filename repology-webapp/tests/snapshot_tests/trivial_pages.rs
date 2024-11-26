// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_trivial_pages(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/api").await;
    uri_snapshot_test(pool.clone(), "/api/v1").await;
    uri_snapshot_test(pool.clone(), "/docs/about").await;
    uri_snapshot_test(pool.clone(), "/docs/bots").await;
    uri_snapshot_test(pool.clone(), "/docs").await;
    uri_snapshot_test(pool.clone(), "/docs/not_supported").await;
    uri_snapshot_test(pool.clone(), "/docs/requirements").await;
    uri_snapshot_test(pool.clone(), "/news").await;
    uri_snapshot_test(pool.clone(), "/tools").await;
}
