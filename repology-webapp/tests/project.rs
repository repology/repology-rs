// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::{check_code, check_html};

// TODO move to test_project_versions when implemented
#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("project_data"))]
async fn test_project_versions_todo(pool: PgPool) {
    check_code!(pool, "/project/nonexistent/versions", NOT_FOUND);
    check_code!(pool, "/project/orphaned/versions", GONE);

    check_html!(
        pool,
        "/project/%3Cmarquee%3E%26nbsp%3B/versions",
        !"<marquee>",
        !"&nbsp;"
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("project_data"))]
async fn test_project_versions(pool: PgPool) {
    check_html!(
        pool,
        "/project/zsh/versions",
        "Versions for <strong>zsh",
        "FreeBSD",
        "1.1"
    );
}
