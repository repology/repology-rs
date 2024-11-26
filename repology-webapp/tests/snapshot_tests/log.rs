// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages", "log_data")
)]
async fn test_log(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/log/10").await;
    uri_snapshot_test(pool.clone(), "/log/foo").await;
    uri_snapshot_test(pool.clone(), "/log/1").await;
    uri_snapshot_test(pool.clone(), "/log/2").await;
}
