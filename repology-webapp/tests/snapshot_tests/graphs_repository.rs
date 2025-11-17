// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_problems_nonexistent_repository(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/unknown/problems.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_problems_legacy_repository(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/ubuntu_10/problems.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_problems(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/problems.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_maintainers(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/maintainers.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_total(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_total.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_unique(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_unique.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_unique_percent(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_unique_percent.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_newest(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_newest.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_newest_percent(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_newest_percent.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_outdated(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_outdated.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_outdated_percent(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_outdated_percent.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_problematic(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_problematic.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_problematic_percent(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_problematic_percent.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_vulnerable(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_vulnerable.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_vulnerable_percent(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_vulnerable_percent.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_problems_per_projects(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/problems_per_1000_projects.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore = "flaky due to dependency on the current time"]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories.sql", "graphs_data.sql"))]
async fn test_projects_per_maintainer(pool: PgPool) {
    let response = Request::new(pool, "/graph/repo/freebsd/projects_per_maintainer.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}
