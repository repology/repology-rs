// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "maintainer_data")
)]
async fn test_maintainer(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/maintainer/nonexistent@example.com").await;
    uri_snapshot_test(pool.clone(), "/maintainer/orphaned@example.com").await;
    uri_snapshot_test(pool.clone(), "/maintainer/orphaned-in-future@example.com").await;
    uri_snapshot_test(pool.clone(), "/maintainer/active@example.com").await;
    uri_snapshot_test(pool.clone(), "/maintainer/fallback-mnt-foo@repology").await;
    uri_snapshot_test(pool.clone(), "/maintainer/no-vuln-column@example.com").await;
}
