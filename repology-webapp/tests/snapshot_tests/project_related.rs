// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/project/nonexistent/related").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_orphaned(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned/related").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_no_relations(pool: PgPool) {
    let response = Request::new(pool, "/project/zsh/related").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_has_relations_a(pool: PgPool) {
    let response = Request::new(pool, "/project/gcc/related").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "related_data"))]
async fn test_has_relations_b(pool: PgPool) {
    let response = Request::new(pool, "/project/binutils/related").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}
