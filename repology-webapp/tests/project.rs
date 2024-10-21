// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_project_versions(pool: PgPool) {
    check_response!(
        pool,
        "/project/nonexistent/versions",
        status NOT_FOUND,
        content_type "text/html",
        contains "Unknown project"
    );
    check_response!(
        pool,
        "/project/orphaned/versions",
        status NOT_FOUND,
        content_type "text/html",
        contains "Gone project"
    );

    check_response!(
        pool,
        "/project/zsh/versions",
        status OK,
        content_type "text/html",
        contains "Versions for <strong>zsh",
        contains "FreeBSD",
        contains "1.1"
    );

    // XSS test
    check_response!(
        pool,
        "/project/%3Cmarquee%3E%26nbsp%3B/versions",
        content_type "text/html",
        contains_not "<marquee>",
        contains_not "&nbsp;"
    );
}
