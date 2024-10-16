// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_data", "log_data")
)]
async fn test_log(pool: PgPool) {
    check_response!(pool, "/log/10", status NOT_FOUND);
    check_response!(pool, "/log/foo", status BAD_REQUEST);

    check_response!(pool, "/log/1", status OK, contains "ongoing");
    check_response!(pool, "/log/2", status OK, contains "successful");
}
