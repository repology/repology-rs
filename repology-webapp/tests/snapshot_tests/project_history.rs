// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_history_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/project/nonexistent/hitstory").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_history_data"))]
async fn test_orphaned_with_history(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned-with-history/history").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_history_data"))]
async fn test_orphaned_without_history(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned-without-history/history").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_history_data"))]
async fn test_normal(pool: PgPool) {
    let response = Request::new(pool, "/project/zsh/history").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}
