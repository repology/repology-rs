// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use insta::assert_snapshot;
use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("project_cves_data"))]
async fn test_recent_cves(pool: PgPool) {
    let response = Request::new(pool, "/security/recent-cves").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("project_cves_data"))]
async fn test_recent_cpes(pool: PgPool) {
    let response = Request::new(pool, "/security/recent-cpes").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
