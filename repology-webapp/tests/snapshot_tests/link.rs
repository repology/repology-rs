// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_nonexistent(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/link/https://example.com/nonexistent").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_not_checked(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/link/https://example.com/not-checked").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv4_failure(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/link/https://example.com/ipv4-failure").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv4_success(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/link/https://example.com/ipv4-success").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv4_redirect(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/link/https://example.com/ipv4-redirect").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv6_failure(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/link/https://example.com/ipv6-failure").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv6_success(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/link/https://example.com/ipv6-success").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv6_redirect(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/link/https://example.com/ipv6-redirect").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ssl_failure(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/link/https://example.com/ssl-failure").await;
}
