// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_missing_extension(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/latest-versions/nonexistent").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_nonexistent(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/latest-versions/nonexistent.svg").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_multiple_versions(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/latest-versions/zsh.svg").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_single_version(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/latest-versions/bash.svg").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_header_custom(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/latest-versions/zsh.svg?header=VERSIONS").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_header_empty(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/latest-versions/zsh.svg?header=").await;
}
