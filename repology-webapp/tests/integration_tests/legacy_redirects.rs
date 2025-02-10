// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_version_only_for_repo(pool: PgPool) {
    let response = Request::new(pool, "/badge/version-only-for-repo/foo/bar.svg").perform().await;
    assert_eq!(response.status(), http::StatusCode::MOVED_PERMANENTLY);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/badge/version-for-repo/foo/bar.svg"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_version_only_for_repo_with_title(pool: PgPool) {
    let response = Request::new(pool, "/badge/version-only-for-repo/foo/bar.svg?header=baz").perform().await;
    assert_eq!(response.status(), http::StatusCode::MOVED_PERMANENTLY);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/badge/version-for-repo/foo/bar.svg?header=baz"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_project_root(pool: PgPool) {
    let response = Request::new(pool, "/project/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::MOVED_PERMANENTLY);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/project/zsh/versions"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_metapackage(pool: PgPool) {
    let response = Request::new(pool, "/metapackage/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::MOVED_PERMANENTLY);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/project/zsh/versions"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_metapackage_versions(pool: PgPool) {
    let response = Request::new(pool, "/metapackage/zsh/versions").perform().await;
    assert_eq!(response.status(), http::StatusCode::MOVED_PERMANENTLY);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/project/zsh/versions"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_metapackage_packages(pool: PgPool) {
    let response = Request::new(pool, "/metapackage/zsh/packages").perform().await;
    assert_eq!(response.status(), http::StatusCode::MOVED_PERMANENTLY);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/project/zsh/packages"));
}
