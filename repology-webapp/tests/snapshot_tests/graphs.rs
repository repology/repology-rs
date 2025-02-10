// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_graphs_total(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/graph/total/packages.svg").await;
    uri_snapshot_test(pool.clone(), "/graph/total/projects.svg").await;
    uri_snapshot_test(pool.clone(), "/graph/total/maintainers.svg").await;
    uri_snapshot_test(pool.clone(), "/graph/total/problems.svg").await;
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_graphs_repository(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/graph/repo/unknown/problems.svg").await;
    uri_snapshot_test(pool.clone(), "/graph/repo/ubuntu_10/problems.svg").await;
    uri_snapshot_test(pool.clone(), "/graph/repo/freebsd/problems.svg").await;
    uri_snapshot_test(pool.clone(), "/graph/repo/freebsd/maintainers.svg").await;
    uri_snapshot_test(pool.clone(), "/graph/repo/freebsd/projects_total.svg").await;
    uri_snapshot_test(pool.clone(), "/graph/repo/freebsd/projects_unique.svg").await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_unique_percent.svg",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/graph/repo/freebsd/projects_newest.svg").await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_newest_percent.svg",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/graph/repo/freebsd/projects_outdated.svg").await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_outdated_percent.svg",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/graph/repo/freebsd/projects_problematic.svg").await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_problematic_percent.svg",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/graph/repo/freebsd/projects_vulnerable.svg").await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_vulnerable_percent.svg",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/problems_per_1000_projects.svg",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_per_maintainer.svg",
    )
    .await;
}
