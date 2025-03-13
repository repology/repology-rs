// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_missing_extension(pool: PgPool) {
    let response = Request::new(pool, "/badge/latest-versions/nonexistent").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/badge/latest-versions/nonexistent.svg").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_multiple_versions(pool: PgPool) {
    let response = Request::new(pool, "/badge/latest-versions/zsh.svg").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_single_version(pool: PgPool) {
    let response = Request::new(pool, "/badge/latest-versions/bash.svg").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_header_custom(pool: PgPool) {
    let response = Request::new(pool, "/badge/latest-versions/zsh.svg?header=VERSIONS").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_header_empty(pool: PgPool) {
    let response = Request::new(pool, "/badge/latest-versions/zsh.svg?header=").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
