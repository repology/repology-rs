// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use anyhow::Error;
use axum::http::{header, Request, StatusCode};
use sqlx::PgPool;
use tower_service::Service;

use repology_webapp::create_app;

struct Response {
    pub status: StatusCode,
    pub content_type: Option<String>,
    pub body: String,
}

async fn get(pool: PgPool, uri: &str) -> Result<Response, Error> {
    let mut app = create_app(pool).await?;
    let response = app
        .call(
            Request::builder()
                .uri(uri)
                .method("GET")
                .body("".to_owned())?,
        )
        .await?;
    Ok(Response {
        status: response.status(),
        content_type: response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok().map(|value| value.into())),
        body: std::str::from_utf8(&axum::body::to_bytes(response.into_body(), 10000).await?)?
            .into(),
    })
}

macro_rules! check_code {
    ($pool:ident, $uri:literal, $code:ident) => {
        let resp = get($pool.clone(), $uri).await.unwrap();
        assert_eq!(resp.status, StatusCode::$code);
    };
}

macro_rules! check_json {
    ($pool:ident, $uri:literal, $expected_json:literal) => {
        let resp = get($pool.clone(), $uri).await.unwrap();
        assert_eq!(resp.status, StatusCode::OK);
        assert_eq!(
            resp.content_type,
            Some(mime::APPLICATION_JSON.as_ref().into())
        );

        let returned = json::stringify_pretty(json::parse(&resp.body).unwrap(), 2);
        let expected = json::stringify_pretty(json::parse($expected_json).unwrap(), 2);
        pretty_assertions::assert_eq!(returned, expected);
    };
}

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
