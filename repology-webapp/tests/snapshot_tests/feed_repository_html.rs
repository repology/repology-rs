// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "repository_feed_data"))]
async fn test_nonexistent_repository(pool: PgPool) {
    let response = Request::new(pool, "/repository/nonexistent/feed").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "repository_feed_data"))]
async fn test_base(pool: PgPool) {
    let response = Request::new(pool, "/repository/freebsd/feed").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}
