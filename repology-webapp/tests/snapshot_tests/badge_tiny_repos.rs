// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_missing_extension(pool: PgPool) {
    let response = Request::new(pool, "/badge/tiny-repos/nonexistent").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/badge/tiny-repos/nonexistent.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_base(pool: PgPool) {
    let response = Request::new(pool, "/badge/tiny-repos/zsh.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_header_custom(pool: PgPool) {
    let response = Request::new(pool, "/badge/tiny-repos/zsh.svg?header=Repository+Count").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_header_empty(pool: PgPool) {
    let response = Request::new(pool, "/badge/tiny-repos/zsh.svg?header=").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}
