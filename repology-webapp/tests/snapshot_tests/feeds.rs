// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "repository_feed_data"))]
async fn test_repository_feed(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/repository/nonexistent/feed").await;
    uri_snapshot_test(pool.clone(), "/repository/freebsd/feed").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "repository_feed_data"))]
async fn test_repository_feed_atom(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/repository/nonexistent/feed/atom").await;
    uri_snapshot_test(pool.clone(), "/repository/freebsd/feed/atom").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "common_maintainers", "maintainer_feed_data"))]
async fn test_maintainer_repo_feed(pool: PgPool) {
    uri_snapshot_test(
        pool.clone(),
        "/maintainer/nonexistent@example.com/feed-for-repo/freebsd",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/maintainer/johndoe@example.com/feed-for-repo/nonexistent",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/maintainer/johndoe@example.com/feed-for-repo/freebsd",
    )
    .await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "common_maintainers", "maintainer_feed_data"))]
async fn test_maintainer_repo_feed_atom(pool: PgPool) {
    uri_snapshot_test(
        pool.clone(),
        "/maintainer/nonexistent@example.com/feed-for-repo/freebsd/atom",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/maintainer/johndoe@example.com/feed-for-repo/nonexistent/atom",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/maintainer/johndoe@example.com/feed-for-repo/freebsd/atom",
    )
    .await;
}
