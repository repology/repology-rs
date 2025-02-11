// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use serde_json::json;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/project/nonexistent").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/json"));
    pretty_assertions::assert_eq!(response.json().unwrap(), json!([]));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_full(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/project/full").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/json"));
    pretty_assertions::assert_eq!(
        response.json().unwrap(),
        json!(
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
        )
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_minimal(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/project/minimal").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/json"));
    pretty_assertions::assert_eq!(
        response.json().unwrap(),
        json!(
        [
            {
                "repo": "repo",
                "visiblename": "visiblename",
                "version": "1.0",
                "origversion": "1.0_1",
                "status": "newest"
            }
        ]
        )
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("api_data"))]
async fn test_vulnerable(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/project/vulnerable").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/json"));
    pretty_assertions::assert_eq!(
        response.json().unwrap(),
        json!(
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
        )
    );
}
