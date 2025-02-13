// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_maintainer(pool: PgPool) {
    uri_snapshot_test(pool, "/opensearch/maintainer.xml").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_project(pool: PgPool) {
    uri_snapshot_test(pool, "/opensearch/project.xml").await;
}
