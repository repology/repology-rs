// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::{check_code, check_html};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("log_data"))]
async fn test_log(pool: PgPool) {
    check_code!(pool, "/log/10", NOT_FOUND);
    check_code!(pool, "/log/foo", BAD_REQUEST);

    check_html!(pool, "/log/1", "ongoing");
    check_html!(pool, "/log/2", "successful");
}
