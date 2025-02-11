// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use serde_json::json;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_from(pool: PgPool) {
    // just a small subset of conditions tested, see more tests in tests/projects.rs
    let response = Request::new(pool, "/api/v1/projects/pkg_foo/").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/json"));
    pretty_assertions::assert_eq!(
        response.json().unwrap(),
        json!(
            {
                "pkg_foofoo_": [
                    {
                        "repo": "ubuntu_12",
                        "visiblename": "",
                        "version": "",
                        "origversion": null,
                        "status": "newest"
                    }
                ]
            }
        )
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/projects/?search=bar").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/json"));
    pretty_assertions::assert_eq!(
        response.json().unwrap(),
        json!(
            {
                "pkg_barbar_": [
                    {
                        "repo": "ubuntu_12",
                        "visiblename": "",
                        "version": "",
                        "origversion": null,
                        "status": "newest"
                    }
                ]
            }
        )
    );
}
