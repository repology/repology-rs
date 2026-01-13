// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/nonexistent@example.com").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_orphaned(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/orphaned@example.com").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_orphaned_in_future(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/orphaned-in-future@example.com").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_active(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/active@example.com").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_fallback(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/fallback-mnt-foo@repology").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_no_vuln_column(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/no-vuln-column@example.com").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}
