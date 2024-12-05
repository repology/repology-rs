// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("project_cves_data"))]
async fn test_recent_cves(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/security/recent-cves").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("project_cves_data"))]
async fn test_recent_cpes(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/security/recent-cpes").await;
}
