// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages", "feeds_data")
)]
async fn test_problems(pool: PgPool) {
    check_response!(
        pool,
        "/repository/nonexistent/feed",
        status NOT_FOUND,
    );
    check_response!(
        pool,
        "/repository/freebsd/feed",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",

        contains ">111<",
        contains ">222<",
        contains ">333<",
        contains ">444<",
        contains ">555<",
        contains ">666<",
    );
}
