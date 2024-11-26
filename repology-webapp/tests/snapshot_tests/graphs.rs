// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_graphs_total(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/graph/total/packages.svg").await;
    uri_snapshot_test(pool.clone(), "/graph/total/projects.svg").await;
    uri_snapshot_test(pool.clone(), "/graph/total/maintainers.svg").await;
    uri_snapshot_test(pool.clone(), "/graph/total/problems.svg").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories.sql", "graphs_data.sql")
)]
async fn test_graphs_repository(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/graph/repo/unknown/problems.svg").await;
    uri_snapshot_test(pool.clone(), "/graph/repo/ubuntu_10/problems.svg").await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/problems.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/maintainers.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_total.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_unique.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_unique_percent.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_newest.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_newest_percent.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_outdated.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_outdated_percent.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_problematic.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_problematic_percent.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_vulnerable.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_vulnerable_percent.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/problems_per_1000_projects.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_per_maintainer.svg?experimental_history=",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/problems.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/maintainers.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_total.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_unique.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_unique_percent.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_newest.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_newest_percent.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_outdated.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_outdated_percent.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_problematic.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_problematic_percent.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_vulnerable.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_vulnerable_percent.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/problems_per_1000_projects.svg?experimental_history=1",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/graph/repo/freebsd/projects_per_maintainer.svg?experimental_history=1",
    )
    .await;
}
