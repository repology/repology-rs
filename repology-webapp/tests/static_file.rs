// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::{check_binary, check_code};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("log_data"))]
async fn test_log(pool: PgPool) {
    check_code!(pool, "/static/nonexistent", NOT_FOUND);
    check_code!(pool, "/static/repology.v1.ico", NOT_FOUND);

    check_binary!(
        pool,
        "/static/repology.v1.6108dff405ea1a42.ico",
        "application/x-icon",
        22382,
        0x6108dff405ea1a42
    );
}
