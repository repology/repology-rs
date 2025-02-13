// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_problems_nonexistent_repository(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/unknown/problems.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_problems_legacy_repository(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/ubuntu_10/problems.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_problems(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/problems.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_maintainers(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/maintainers.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_total(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/projects_total.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_unique(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/projects_unique.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_unique_percent(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/projects_unique_percent.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_newest(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/projects_newest.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_newest_percent(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/projects_newest_percent.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_outdated(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/projects_outdated.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_outdated_percent(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/projects_outdated_percent.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_problematic(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/projects_problematic.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_problematic_percent(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/projects_problematic_percent.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_vulnerable(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/projects_vulnerable.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_vulnerable_percent(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/projects_vulnerable_percent.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_problems_per_projects(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/problems_per_1000_projects.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_per_maintainer(pool: PgPool) {
    uri_snapshot_test(pool, "/graph/repo/freebsd/projects_per_maintainer.svg").await;
}
