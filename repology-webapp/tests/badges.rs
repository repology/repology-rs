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

macro_rules! check_svg {
    ($pool:ident, $uri:literal $(, $($has:literal)? $(!$hasnt:literal)? )*) => {
        let resp = get($pool.clone(), $uri)
            .await
            .unwrap();
        assert!(sxd_document::parser::parse(&resp.body).is_ok(), "failed to parse XML document");
        assert_eq!(resp.status, StatusCode::OK);
        assert_eq!(resp.content_type, Some(mime::IMAGE_SVG.as_ref().into()));

        $(
            $(
                assert!(resp.body.contains($has));
            )?
            $(
                assert!(!resp.body.contains($hasnt));
            )?
        )*
    };
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_data"))]
async fn test_badge_tiny_repos(pool: PgPool) {
    check_code!(pool, "/badge/tiny-repos/nonexistent", NOT_FOUND);
    check_svg!(
        pool,
        "/badge/tiny-repos/nonexistent.svg",
        "in repositories",
        ">0<"
    );
    check_svg!(pool, "/badge/tiny-repos/zsh.svg", "in repositories", ">2<");
    check_svg!(
        pool,
        "/badge/tiny-repos/zsh.svg?header=Repository+Count",
        !"in repositories",
        "Repository Count",
        ">2<"
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_data"))]
async fn test_badge_version_for_repo(pool: PgPool) {
    check_code!(pool, "/badge/version_for_repo/freebsd/zsh", NOT_FOUND);
    check_code!(pool, "/badge/version_for_repo/badrepo/zsh", NOT_FOUND);
    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg",
        "FreeBSD port",
        ">1.1<",
        "#4c1",
    );
    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg?minversion=1.2",
        "FreeBSD port",
        ">1.1<",
        "#e00000",
    );
    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg?header=fbsd+ver",
        !"FreeBSD port",
        "fbsd ver",
    );
    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/unpackaged.svg",
        ">-<",
    );
    check_svg!(
        pool,
        "/badge/version-for-repo/ubuntu/zsh.svg",
        ">1.0<",
        "#e05d44",
    );
    check_svg!(
        pool,
        "/badge/version-for-repo/ubuntu/zsh.svg?allow_ignored=1",
        ">1.2<",
        "#9f9f9f",
    );
}
