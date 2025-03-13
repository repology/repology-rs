// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "log_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/log/10").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "log_data"))]
async fn test_invalid_id(pool: PgPool) {
    let response = Request::new(pool, "/log/foo").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "log_data"))]
async fn test_ongoing(pool: PgPool) {
    let response = Request::new(pool, "/log/1").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "log_data"))]
async fn test_finished(pool: PgPool) {
    let response = Request::new(pool, "/log/2").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
