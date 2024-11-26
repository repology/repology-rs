// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_legacy_redirects(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/badge/version-only-for-repo/foo/bar.svg").await;
    uri_snapshot_test(
        pool.clone(),
        "/badge/version-only-for-repo/foo/bar.svg?header=baz",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/project/zsh").await;
    uri_snapshot_test(pool.clone(), "/metapackage/zsh").await;
    uri_snapshot_test(pool.clone(), "/metapackage/zsh/versions").await;
    uri_snapshot_test(pool.clone(), "/metapackage/zsh/packages").await;
}
