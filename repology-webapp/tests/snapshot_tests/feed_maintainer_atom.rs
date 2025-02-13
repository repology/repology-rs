// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "common_maintainers", "maintainer_feed_data"))]
async fn test_nonexisitent_maintainer(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainer/nonexistent@example.com/feed-for-repo/freebsd/atom").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "common_maintainers", "maintainer_feed_data"))]
async fn test_nonexistent_repository(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainer/johndoe@example.com/feed-for-repo/nonexistent/atom").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "common_maintainers", "maintainer_feed_data"))]
async fn test_base(pool: PgPool) {
    uri_snapshot_test(pool, "/maintainer/johndoe@example.com/feed-for-repo/freebsd/atom").await;
}
