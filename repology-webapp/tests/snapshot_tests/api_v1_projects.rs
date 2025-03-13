// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_from(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/projects/pkg_foo/").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/projects/?search=bar").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
