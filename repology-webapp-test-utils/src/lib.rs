// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod __private {
    pub use axum;
    pub use bytes;
    pub use json;
    pub use mime;
    pub use pretty_assertions;
    pub use regex;
    pub use sxd_document;
    pub use sxd_xpath;

    use tower_service::Service;

    use repology_webapp::create_app;

    #[derive(Debug)]
    pub struct Response {
        pub status: axum::http::StatusCode,
        pub content_type: Option<String>,
        pub body: bytes::Bytes,
        pub text: std::cell::OnceCell<String>,
        pub xml: std::cell::OnceCell<sxd_document::Package>,
    }

    impl Response {
        pub fn text(&self) -> &String {
            self.text.get_or_init(|| {
                std::str::from_utf8(&self.body)
                    .expect("response should be valid utf-8 text")
                    .into()
            })
        }

        pub fn xml(&self) -> sxd_document::dom::Document {
            self.xml
                .get_or_init(|| {
                    sxd_document::parser::parse(self.text())
                        .expect("response should be a parsable XML document")
                })
                .as_document()
        }
    }

    pub async fn get(
        pool: sqlx::PgPool,
        uri: &str,
        headers: &[(&str, &str)],
    ) -> Result<Response, anyhow::Error> {
        let mut request = axum::http::Request::builder().uri(uri).method("GET");
        for &(k, v) in headers {
            request = request.header(k, v);
        }
        let request = request.body("".to_owned())?;

        let mut app = create_app(pool).await?;
        let response = app.call(request).await?;
        Ok(Response {
            status: response.status(),
            content_type: response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok().map(|value| value.into())),
            body: axum::body::to_bytes(response.into_body(), 1000000).await?,
            text: std::cell::OnceCell::new(),
            xml: std::cell::OnceCell::new(),
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
macro_rules! check_condition {
    ($resp:ident, status, $status:literal) => {
        assert_eq!($resp.status, $status);
    };
    ($resp:ident, status, $status:ident) => {
        assert_eq!(
            $resp.status,
            $crate::__private::axum::http::StatusCode::$status
        );
    };
    ($resp:ident, content_type, $content_type:literal) => {
        assert_eq!(
            $resp.content_type.as_ref().map(|s| s.as_str()),
            Some($content_type)
        );
    };
    ($resp:ident, content_type, $content_type:ident) => {
        assert_eq!(
            $resp.content_type.as_ref().map(|s| s.as_str()),
            Some($crate::__private::mime::$content_type.as_ref().into())
        );
    };
    ($resp:ident, contains, $text:literal) => {
        assert!($resp.text().contains($text));
    };
    ($resp:ident, contains_not, $text:literal) => {
        assert!(!$resp.text().contains($text));
    };
    ($resp:ident, body_length, $length:literal) => {
        assert_eq!($resp.body.len(), $length);
    };
    ($resp:ident, body_cityhash64, $hash:literal) => {
        assert_eq!(cityhasher::hash::<u64>(&$resp.body), $hash);
    };
    ($resp:ident, json, $json:literal) => {
        let returned = $crate::__private::json::stringify_pretty(
            $crate::__private::json::parse($resp.text())
                .expect("failed to parse json from response"),
            4,
        );
        let expected = $crate::__private::json::stringify_pretty(
            $crate::__private::json::parse($json).expect("failed to parse expected json"),
            4,
        );
        $crate::__private::pretty_assertions::assert_eq!(returned, expected);
    };
    ($resp:ident, svg_xpath, $xpath:literal, $value:literal) => {{
        use $crate::__private::EqualsToXpathValue;

        let factory = $crate::__private::sxd_xpath::Factory::new();
        let xpath = factory
            .build($xpath)
            .expect("failed to parse xpath")
            .expect("no xpath");

        let mut context = $crate::__private::sxd_xpath::Context::new();
        context.set_namespace("svg", "http://www.w3.org/2000/svg");

        let xpath_res = xpath
            .evaluate(&context, $resp.xml().root())
            .expect("failed to evaluate xpath");
        assert!(
            $value.equals_to_xpath_value(&xpath_res),
            "unexpected xpath value {:?} while expected \"{}\"",
            xpath_res,
            $value
        );
    }};
    ($resp:ident, matches, $regexp:literal) => {
        assert!($crate::__private::regex::Regex::new($regexp)
            .expect("failed to parse regex")
            .is_match($resp.text()));
    };
    ($resp:ident, matches_not, $regexp:literal) => {
        assert!(!$crate::__private::regex::Regex::new($regexp)
            .expect("failed to parse regex")
            .is_match($resp.text()));
    };
    ($resp:ident, line_matches, $regexp:literal) => {
        let re = $crate::__private::regex::Regex::new($regexp).expect("failed to parse regex");
        assert!($resp.text().lines().any(|line| re.is_match(line)));
    };
    ($resp:ident, line_matches_not, $regexp:literal) => {
        let re = $crate::__private::regex::Regex::new($regexp).expect("failed to parse regex");
        assert!(!$resp.text().lines().any(|line| re.is_match(line)));
    };
}

#[macro_export]
macro_rules! check_response {
    (
        $pool:ident,
        $(add_header $header_name:literal $header_value:literal, )*
        $uri:literal
        $(, $cond:tt $value:tt $($extra_value:literal)?)*
        $(,)?
    ) => {
        let resp = $crate::__private::get($pool.clone(), $uri, &[
            $(
                ($header_name, $header_value),
            )*
        ]).await.expect("request to web application failed");
        dbg!(&resp);
        $(
            $crate::check_condition!(resp, $cond, $value $(, $extra_value)?);
        )*
    };
}
