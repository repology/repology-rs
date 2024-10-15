// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::{check_code, check_html, check_html2};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("project_data"))]
async fn test_project_versions(pool: PgPool) {
    check_html2!(pool, "/project/nonexistent/versions", status NOT_FOUND, contains "Unknown project");
    check_html2!(pool, "/project/orphaned/versions", status NOT_FOUND, contains "Gone project");

    check_html2!(
        pool,
        "/project/zsh/versions",
        status OK,
        contains "Versions for <strong>zsh",
        contains "FreeBSD",
        contains "1.1"
    );

    // XSS test
    check_html2!(
        pool,
        "/project/%3Cmarquee%3E%26nbsp%3B/versions",
        !contains "<marquee>",
        !contains "&nbsp;"
    );
}
