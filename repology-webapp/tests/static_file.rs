// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("log_data"))]
async fn test_log(pool: PgPool) {
    check_response!(pool, "/static/nonexistent", status NOT_FOUND);
    check_response!(pool, "/static/repology.v1.ico", status NOT_FOUND);

    check_response!(
        pool,
        "/static/repology.v1.6108dff405ea1a42.ico",
        status OK,
        content_type "application/x-icon",
        body_length 22382,
        body_cityhash64 0x6108dff405ea1a42,
    );
    check_response!(
        pool,
        add_header "accept-encoding" "gzip",
        "/static/repology.v1.6108dff405ea1a42.ico",
        status OK,
        content_type "application/x-icon",
        body_length 3117,
        body_cityhash64 10174067632225889947
    );
    check_response!(
        pool,
        add_header "accept-encoding" "br;q=1.0, gzip;q=0.8, *;q=0.1",
        "/static/repology.v1.6108dff405ea1a42.ico",
        status OK,
        content_type "application/x-icon",
        body_length 3117,
        body_cityhash64 10174067632225889947
    );
}
