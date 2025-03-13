// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_base(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=zsh,fish").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_require_all(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=zsh,fish&require_all=1").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_little_repos(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=fish").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_force_missing_repo(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=fish&repos=freebsd,ubuntu_24").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_header(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=zsh,fish&header=Custom%20header").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_exclude_unsupported(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=zsh,fish&exclude_unsupported=1").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_exclude_sources(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=zsh,fish&exclude_sources=site").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_limit_version(pool: PgPool) {
    let response = Request::new(pool, "/badge/versions-matrix.svg?projects=zsh%3C1.1,fish").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
