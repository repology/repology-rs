// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_json;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_badge_tiny_repos(pool: PgPool) {
    check_json!(pool, "/api/v1/project/nonexistent", "[]");
    check_json!(
        pool,
        "/api/v1/project/full",
        r#"
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
    check_json!(
        pool,
        "/api/v1/project/minimal",
        r#"
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
    check_json!(
        pool,
        "/api/v1/project/vulnerable",
        r#"
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
