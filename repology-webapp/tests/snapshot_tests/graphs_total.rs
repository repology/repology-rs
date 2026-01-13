// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_packages(pool: PgPool) {
    let response = Request::new(pool, "/graph/total/packages.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_projects(pool: PgPool) {
    let response = Request::new(pool, "/graph/total/projects.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_maintainers(pool: PgPool) {
    let response = Request::new(pool, "/graph/total/maintainers.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("graphs_data.sql"))]
async fn test_problems(pool: PgPool) {
    let response = Request::new(pool, "/graph/total/problems.svg").perform().await;
    assert_snapshot!(response.as_text_snapshot().unwrap());
}
