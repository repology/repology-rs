// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_api_v1_project(pool: PgPool) {
    check_response!(pool,
        "/api/v1/project/nonexistent",
        content_type "application/json",
        json "[]"
    );
    check_response!(
        pool,
        "/api/v1/project/full",
        content_type "application/json",
        json r#"
        [
            {
                "repo": "repo",
                "subrepo": "subrepo",
                "srcname": "srcname",
                "binname": "binname",
                "visiblename": "visiblename",
                "version": "1.0",
                "origversion": "1.0_1",
                "status": "newest",
                "summary": "Summary",
                "maintainers": [
                    "foo@example.com",
                    "bar@example.com"
                ],
                "licenses": [
                    "GPLv2",
                    "GPLv3"
                ],
                "categories": [
                    "games"
                ]
            }
        ]
        "#
    );
    check_response!(
        pool,
        "/api/v1/project/minimal",
        content_type "application/json",
        json r#"
        [
            {
                "repo": "repo",
                "visiblename": "visiblename",
                "version": "1.0",
                "origversion": "1.0_1",
                "status": "newest"
            }
        ]
        "#
    );
    check_response!(
        pool,
        "/api/v1/project/vulnerable",
        content_type "application/json",
        json r#"
        [
            {
                "repo": "repo",
                "visiblename": "visiblename",
                "version": "1.0",
                "origversion": "1.0_1",
                "status": "newest",
                "vulnerable": true
            }
        ]
        "#
    );
}
