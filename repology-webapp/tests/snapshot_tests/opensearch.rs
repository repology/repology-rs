// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_opensearch(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/opensearch/maintainer.xml").await;
    uri_snapshot_test(pool.clone(), "/opensearch/project.xml").await;
}
