// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/project/nonexistent/versions-compact").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_orphaned(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned/versions-compact").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_normal(pool: PgPool) {
    let response = Request::new(pool, "/project/zsh/versions-compact").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_xxx_attempt(pool: PgPool) {
    let response = Request::new(pool, "/project/%3Cmarquee%3E%26nbsp%3B/versions-compact").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
