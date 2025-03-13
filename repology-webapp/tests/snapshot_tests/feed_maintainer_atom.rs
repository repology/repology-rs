// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "common_maintainers", "maintainer_feed_data"))]
async fn test_nonexisitent_maintainer(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/nonexistent@example.com/feed-for-repo/freebsd/atom").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "common_maintainers", "maintainer_feed_data"))]
async fn test_nonexistent_repository(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/johndoe@example.com/feed-for-repo/nonexistent/atom").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "common_maintainers", "maintainer_feed_data"))]
async fn test_base(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/johndoe@example.com/feed-for-repo/freebsd/atom").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
