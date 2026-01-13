// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_missing_extension(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/nonexistent").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_nonexistent_repository(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/nonexistent.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_legacy_repository(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/ubuntu_10.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_active_repository_without_packages(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/freshcode.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_active_repository(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/freebsd.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_header_custom(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/freebsd.svg?header=FreeBSD").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_header_empty(pool: PgPool) {
    let response = Request::new(pool, "/badge/repository-big/freebsd.svg?header=").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}
