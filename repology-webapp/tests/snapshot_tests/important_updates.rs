// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use insta::assert_snapshot;
use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("important_updates"))]
async fn test_base(pool: PgPool) {
    let response = Request::new(pool, "/tools/important-updates").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
