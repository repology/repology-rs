// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_packages(pool: PgPool) {
    uri_snapshot_test(pool, "/repositories/packages").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_updates(pool: PgPool) {
    uri_snapshot_test(pool, "/repositories/updates").await;
}
