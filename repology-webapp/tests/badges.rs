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

trait EqualsToXpathValue {
    fn equals_to_xpath_value(&self, v: &sxd_xpath::Value) -> bool;
}

impl EqualsToXpathValue for bool {
    fn equals_to_xpath_value(&self, v: &sxd_xpath::Value) -> bool {
        match v {
            sxd_xpath::Value::Boolean(b) => b == self,
            _ => false,
        }
    }
}

impl EqualsToXpathValue for &str {
    fn equals_to_xpath_value(&self, v: &sxd_xpath::Value) -> bool {
        match v {
            sxd_xpath::Value::String(s) => s == self,
            _ => false,
        }
    }
}

impl EqualsToXpathValue for f64 {
    fn equals_to_xpath_value(&self, v: &sxd_xpath::Value) -> bool {
        match v {
            sxd_xpath::Value::Number(f) => f == self,
            _ => false,
        }
    }
}

macro_rules! check_svg {
    ($pool:ident, $uri:literal $(, $($has:literal)? $(!$hasnt:literal)? $(@$xpath_expr:literal==$xpath_value:literal)?)*) => {
        let resp = get($pool.clone(), $uri)
            .await
            .unwrap();
        assert_eq!(resp.status, StatusCode::OK);
        assert_eq!(resp.content_type, Some(mime::IMAGE_SVG.as_ref().into()));

        let parsed = sxd_document::parser::parse(&resp.body);
        assert!(parsed.is_ok(), "failed to parse XML document");
        let parsed = parsed.unwrap();
        let _document = parsed.as_document();

        $(
            $(
                assert!(resp.body.contains($has));
            )?
            $(
                assert!(!resp.body.contains($hasnt));
            )?
            $(
                {
                    let factory = sxd_xpath::Factory::new();
                    let xpath = factory.build($xpath_expr).unwrap();
                    let xpath = xpath.unwrap();

                    let mut context = sxd_xpath::Context::new();
                    context.set_namespace("svg", "http://www.w3.org/2000/svg");

                    let xpath_res = xpath.evaluate(&context, _document.root()).unwrap();
                    assert!($xpath_value.equals_to_xpath_value(&xpath_res), "unexpected xpath value {:?} while expected \"{}\"", xpath_res, $xpath_value);
                }
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
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 4_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "in repositories",
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "0",
    );
    check_svg!(pool, "/badge/tiny-repos/zsh.svg",
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 4_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "in repositories",
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "3",
    );

    // caption flags
    check_svg!(
        pool,
        "/badge/tiny-repos/zsh.svg?header=Repository+Count",
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "Repository Count",
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "3",
    );
    check_svg!(
        pool,
        "/badge/tiny-repos/zsh.svg?header=",
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 2_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "3",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_data"))]
async fn test_badge_version_for_repo(pool: PgPool) {
    check_code!(pool, "/badge/version-for-repo/freebsd/zsh", NOT_FOUND);
    check_code!(pool, "/badge/version-for-repo/badrepo/zsh.svg", NOT_FOUND);
    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg",
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 4_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "FreeBSD port",
        @"count(//svg:g[1]/svg:rect[@fill='#4c1'])" == 1_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "1.1",
    );

    // minversion_flag
    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg?minversion=1.2",
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "FreeBSD port",
        @"count(//svg:g[1]/svg:rect[@fill='#e00000'])" == 1_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "1.1",
    );

    // caption flags
    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg?header=fbsd+ver",
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "fbsd ver",
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "1.1",
    );
    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg?header=",
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 2_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "1.1",
    );

    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/unpackaged.svg",
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "FreeBSD port",
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "-",
    );
    check_svg!(
        pool,
        "/badge/version-for-repo/ubuntu_24/zsh.svg",
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "Ubuntu 24 package",
        @"count(//svg:g[1]/svg:rect[@fill='#e05d44'])" == 1_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "1.0",
    );
    check_svg!(
        pool,
        "/badge/version-for-repo/ubuntu_24/zsh.svg?allow_ignored=1",
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "Ubuntu 24 package",
        @"count(//svg:g[1]/svg:rect[@fill='#9f9f9f'])" == 1_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "1.2",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_data"))]
async fn test_badge_vertical_allrepos(pool: PgPool) {
    check_code!(pool, "/badge/vertical-allrepos/zsh", NOT_FOUND);
    check_svg!(
        pool,
        "/badge/vertical-allrepos/unpackaged.svg",
        @"string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" == "No known packages",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/unpackaged.svg?header=Packages",
        @"string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" == "Packages",
    );

    // version flags
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" == "FreeBSD",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[3])" == "1.1",
        @"string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])" == "freshcode.club",
        @"string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[3])" == "1.0",
        @"string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])" == "Ubuntu 12",
        @"string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[3])" == "0.9",
        @"string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])" == "Ubuntu 24",
        @"string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[3])" == "1.0",
        @"string(//svg:g[1]/svg:rect[2]/@fill)" == "#4c1",
        @"string(//svg:g[1]/svg:rect[4]/@fill)" == "#e05d44",
        @"string(//svg:g[1]/svg:rect[6]/@fill)" == "#e05d44",
        @"string(//svg:g[1]/svg:rect[8]/@fill)" == "#e05d44",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?allow_ignored=1",
        @"string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])" == "Ubuntu 24",
        @"string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[3])" == "1.2",
        @"string(//svg:g[1]/svg:rect[8]/@fill)" == "#9f9f9f",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?minversion=1.0",
        @"string(//svg:g[1]/svg:rect[2]/@fill)" == "#4c1",
        @"string(//svg:g[1]/svg:rect[4]/@fill)" == "#e05d44",
        @"string(//svg:g[1]/svg:rect[6]/@fill)" == "#e00000",
        @"string(//svg:g[1]/svg:rect[8]/@fill)" == "#e05d44",
    );

    // repository filters
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" == "FreeBSD",
        @"string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])" == "freshcode.club",
        @"string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])" == "Ubuntu 12",
        @"string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])" == "Ubuntu 24",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?exclude_unsupported=1",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" == "FreeBSD",
        @"string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])" == "freshcode.club",
        @"string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])" == "Ubuntu 24",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?exclude_sources=site",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" == "FreeBSD",
        @"string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])" == "Ubuntu 12",
        @"string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])" == "Ubuntu 24",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" == "freshcode.club",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository,site",
        @"count(//svg:g[1]/svg:g[@font-size=11]/svg:text)" == 0_f64,
    );
}
