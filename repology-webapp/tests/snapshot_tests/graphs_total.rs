// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_packages(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/total/packages.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_projects(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/total/projects.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_maintainers(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/total/maintainers.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_problems(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/total/problems.svg").await;
}
