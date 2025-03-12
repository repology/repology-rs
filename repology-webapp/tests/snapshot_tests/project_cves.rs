// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use insta::assert_snapshot;
use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/project/nonexistent/cves").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_orphaned_without_cves(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned-without-cves/cves").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_orphaned_with_cves(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned-with-cves/cves").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_ranges(pool: PgPool) {
    let response = Request::new(pool, "/project/manyranges/cves").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_version_not_highlighted(pool: PgPool) {
    let response = Request::new(pool, "/project/tworanges/cves").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_open_range_version_not_highlighted(pool: PgPool) {
    let response = Request::new(pool, "/project/tworanges/cves?version=1.3").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_cves_data"))]
async fn test_version_highlighted(pool: PgPool) {
    let response = Request::new(pool, "/project/tworanges/cves?version=1.4").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
