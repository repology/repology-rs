// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod __private {
    pub use axum;
    pub use json;
    pub use mime;
    pub use pretty_assertions;
    pub use sxd_document;
    pub use sxd_xpath;

    use tower_service::Service;

    use repology_webapp::create_app;

    pub struct Response {
        pub status: axum::http::StatusCode,
        pub content_type: Option<String>,
        pub body: String,
    }

    pub async fn get(pool: sqlx::PgPool, uri: &str) -> Result<Response, anyhow::Error> {
        let mut app = create_app(pool).await?;
        let response = app
            .call(
                axum::http::Request::builder()
                    .uri(uri)
                    .method("GET")
                    .body("".to_owned())?,
            )
            .await?;
        Ok(Response {
            status: response.status(),
            content_type: response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok().map(|value| value.into())),
            body: std::str::from_utf8(&axum::body::to_bytes(response.into_body(), 10000).await?)?
                .into(),
        })
    }

    pub trait EqualsToXpathValue {
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
}

#[macro_export]
macro_rules! check_code {
    ($pool:ident, $uri:literal, $code:ident) => {
        let resp = $crate::__private::get($pool.clone(), $uri).await.unwrap();
        assert_eq!(
            resp.status,
            $crate::__private::axum::http::StatusCode::$code
        );
    };
}

#[macro_export]
macro_rules! check_json {
    ($pool:ident, $uri:literal, $expected_json:literal) => {
        let resp = $crate::__private::get($pool.clone(), $uri).await.unwrap();
        assert_eq!(resp.status, $crate::__private::axum::http::StatusCode::OK);
        assert_eq!(
            resp.content_type,
            Some($crate::__private::mime::APPLICATION_JSON.as_ref().into())
        );

        let returned = $crate::__private::json::stringify_pretty(
            $crate::__private::json::parse(&resp.body).unwrap(),
            4,
        );
        let expected = $crate::__private::json::stringify_pretty(
            $crate::__private::json::parse($expected_json).unwrap(),
            4,
        );
        $crate::__private::pretty_assertions::assert_eq!(returned, expected);
    };
}

#[macro_export]
macro_rules! check_svg {
    ($pool:ident, $uri:literal $(, $($has:literal)? $(!$hasnt:literal)? $(@$xpath_expr:literal==$xpath_value:literal)?)*) => {
        let resp = $crate::__private::get($pool.clone(), $uri)
            .await
            .unwrap();
        assert_eq!(resp.status, $crate::__private::axum::http::StatusCode::OK);
        assert_eq!(resp.content_type, Some($crate::__private::mime::IMAGE_SVG.as_ref().into()));

        let parsed = $crate::__private::sxd_document::parser::parse(&resp.body);
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
                    use $crate::__private::EqualsToXpathValue;

                    let factory = $crate::__private::sxd_xpath::Factory::new();
                    let xpath = factory.build($xpath_expr).unwrap();
                    let xpath = xpath.unwrap();

                    let mut context = $crate::__private::sxd_xpath::Context::new();
                    context.set_namespace("svg", "http://www.w3.org/2000/svg");

                    let xpath_res = xpath.evaluate(&context, _document.root()).unwrap();
                    assert!($xpath_value.equals_to_xpath_value(&xpath_res), "unexpected xpath value {:?} while expected \"{}\"", xpath_res, $xpath_value);
                }
            )?
        )*
    };
}

#[macro_export]
macro_rules! check_html {
    ($pool:ident, $uri:literal $(, $($has:literal)? $(!$hasnt:literal)?)*) => {
        let resp = $crate::__private::get($pool.clone(), $uri)
            .await
            .unwrap();
        assert_eq!(resp.status, $crate::__private::axum::http::StatusCode::OK);
        assert_eq!(resp.content_type, Some($crate::__private::mime::TEXT_HTML.as_ref().into()));

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
