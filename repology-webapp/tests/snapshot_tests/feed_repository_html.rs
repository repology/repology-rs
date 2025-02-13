// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "repository_feed_data"))]
async fn test_nonexistent_repository(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/repository/nonexistent/feed").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "repository_feed_data"))]
async fn test_base(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/repository/freebsd/feed").await;
}
