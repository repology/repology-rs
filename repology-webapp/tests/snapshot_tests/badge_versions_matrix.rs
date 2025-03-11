// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_base(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/versions-matrix.svg?projects=zsh,fish").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_require_all(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/versions-matrix.svg?projects=zsh,fish&require_all=1").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_little_repos(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/versions-matrix.svg?projects=fish").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_force_missing_repo(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/versions-matrix.svg?projects=fish&repos=freebsd,ubuntu_24").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_header(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/versions-matrix.svg?projects=zsh,fish&header=Custom%20header").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_exclude_unsupported(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/versions-matrix.svg?projects=zsh,fish&exclude_unsupported=1").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_exclude_sources(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/versions-matrix.svg?projects=zsh,fish&exclude_sources=site").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_limit_version(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/versions-matrix.svg?projects=zsh%3C1.1,fish").await;
}
