// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_maintainer(pool: PgPool) {
    let response = Request::new(pool, "/opensearch/maintainer.xml").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_project(pool: PgPool) {
    let response = Request::new(pool, "/opensearch/project.xml").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
