// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_json_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_repository(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/repository/freebsd/problems").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/json"));
    insta::with_settings!({sort_maps => true}, {
        assert_json_snapshot!(response.json().unwrap());
    });
}

// we only have one project in fixtures atm, so we can only check if it fits or is missed from the page
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_repository_pagination_a(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/repository/freebsd/problems?start=z").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/json"));
    insta::with_settings!({sort_maps => true}, {
        assert_json_snapshot!(response.json().unwrap());
    });
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_repository_pagination_b(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/repository/freebsd/problems?start=zzz").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/json"));
    insta::with_settings!({sort_maps => true}, {
        assert_json_snapshot!(response.json().unwrap());
    });
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_maintainer(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/maintainer/johndoe@example.com/problems-for-repo/freebsd").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/json"));
    insta::with_settings!({sort_maps => true}, {
        assert_json_snapshot!(response.json().unwrap());
    });
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_maintainer_pagination_a(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/maintainer/johndoe@example.com/problems-for-repo/freebsd?start=z").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/json"));
    insta::with_settings!({sort_maps => true}, {
        assert_json_snapshot!(response.json().unwrap());
    });
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "problems_data"))]
async fn test_maintainer_pagination_b(pool: PgPool) {
    let response = Request::new(pool, "/api/v1/maintainer/johndoe@example.com/problems-for-repo/freebsd?start=zzz").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/json"));
    insta::with_settings!({sort_maps => true}, {
        assert_json_snapshot!(response.json().unwrap());
    });
}
