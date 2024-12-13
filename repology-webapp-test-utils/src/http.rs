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

    pub struct Response {
        pub status: axum::http::StatusCode,
        pub headers: axum::http::header::HeaderMap,
        pub body: bytes::Bytes,
        pub text: Option<String>,
        pub xml: std::cell::OnceCell<sxd_document::Package>,
    }

    impl Response {
        pub fn text(&self) -> &String {
            self.text
                .as_ref()
                .expect("response should be valid utf-8 text")
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
        let status = response.status();
        let headers = response.headers().clone();
        let body = axum::body::to_bytes(response.into_body(), 1000000).await?;
        let text = std::str::from_utf8(&body).ok().map(|s| s.to_owned());
        Ok(Response {
            status,
            headers,
            body,
            text,
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

    impl std::fmt::Debug for Response {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "Response {{")?;
            writeln!(f, "    status: {}", self.status)?;
            for (k, v) in &self.headers {
                writeln!(f, "    header {}: {:?}", k, v)?;
            }

            if let Some(text) = &self.text {
                writeln!(f, "    text:")?;
                for (number, line) in text.lines().enumerate() {
                    writeln!(f, "      {:>5} {}", number + 1, line)?;
                }
            } else {
                writeln!(f, "    body: <binary>")?;
            }
            writeln!(f, "}}")?;
            Ok(())
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
            $resp
                .headers
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some($content_type)
        );
    };
    ($resp:ident, content_type, $content_type:ident) => {
        assert_eq!(
            $resp
                .headers
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some($crate::__private::mime::$content_type.as_ref().into())
        );
    };
    ($resp:ident, contains, $text:literal) => {
        assert!(
            $resp.text().contains($text),
            "contains condition with text \"{}\" has not matched",
            $text
        );
    };
    ($resp:ident, contains_not, $text:literal) => {
        assert!(
            !$resp.text().contains($text),
            "contains_not condition with text \"{}\" has unexpectedly matched",
            $text
        );
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
    ($resp:ident, xpath, $xpath:literal, $value:literal) => {{
        use $crate::__private::EqualsToXpathValue;

        let factory = $crate::__private::sxd_xpath::Factory::new();
        let xpath = factory
            .build($xpath)
            .expect("failed to parse xpath")
            .expect("no xpath");

        let mut context = $crate::__private::sxd_xpath::Context::new();

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
        assert!(
            $crate::__private::regex::Regex::new($regexp)
                .expect("failed to parse regex")
                .is_match($resp.text()),
            "matches condition with regexp \"{}\" has not matched",
            $regexp
        );
    };
    ($resp:ident, matches_not, $regexp:literal) => {
        assert!(
            !$crate::__private::regex::Regex::new($regexp)
                .expect("failed to parse regex")
                .is_match($resp.text()),
            "matches_not condition with regexp \"{}\" has unexpectedly matched",
            $regexp
        );
    };
    ($resp:ident, line_matches, $regexp:literal) => {
        let re = $crate::__private::regex::Regex::new($regexp).expect("failed to parse regex");
        assert!(
            $resp.text().lines().any(|line| re.is_match(line)),
            "line_matches condition with regexp \"{}\" has not matched",
            $regexp
        );
    };
    ($resp:ident, line_matches_not, $regexp:literal) => {
        let re = $crate::__private::regex::Regex::new($regexp).expect("failed to parse regex");
        assert!(
            !$resp.text().lines().any(|line| re.is_match(line)),
            "line_matches_not condition with regexp \"{}\" has unexpectedly matched",
            $regexp
        );
    };
    ($resp:ident, html_ok, $level:literal) => {
        let mut validation_result = $crate::tidy::validate_html($resp.text());
        dbg!(&validation_result);

        let mut allow_warnings = true;
        for param in $level.split(',') {
            match param {
                "warnings_fatal" => {
                    allow_warnings = false;
                }
                "allow_empty_tags" => {
                    validation_result
                        .output
                        .retain(|s| !s.contains("Warning: trimming empty <"));
                }
                "" => {}
                other => {
                    panic!("unknown html_ok flag {}", other);
                }
            }
        }

        assert!(
            validation_result.status == $crate::tidy::ValidationStatus::Ok
                || validation_result.status == $crate::tidy::ValidationStatus::WithWarnings,
            "HTML validation failed"
        );
        if !allow_warnings {
            assert!(
                validation_result.output.is_empty(),
                "HTML validation failed (warnings treated as fatal)"
            );
        }
    };
    ($resp:ident, header_present, $header:literal) => {
        assert!($resp.headers.contains_key($header));
    };
    ($resp:ident, header_value, $header:literal, $value:literal) => {
        let header = $resp.headers.get($header);
        assert!(header.is_some());
        assert_eq!(header.unwrap(), $value);
    };
    ($resp:ident, header_value_contains, $header:literal, $needle:literal) => {
        let header = $resp.headers.get($header);
        assert!(header.is_some_and(|header| { header.to_str().unwrap().contains($needle) }));
    };
    ($resp:ident, header_value_contains_not, $header:literal, $needle:literal) => {
        let header = $resp.headers.get($header);
        assert!(header.is_some_and(|header| { !header.to_str().unwrap().contains($needle) }));
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
        let response = $crate::__private::get($pool.clone(), $uri, &[
            $(
                ($header_name, $header_value),
            )*
        ]).await.expect("request to web application failed");
        dbg!(&response);
        $(
            $crate::check_condition!(response, $cond, $value $(, $extra_value)?);
        )*
    };
    (
        $pool:ident,
        $uri:expr
        $(, $cond:tt $value:tt $($extra_value:literal)?)*
        $(,)?
    ) => {
        let response = $crate::__private::get($pool.clone(), $uri, &[])
            .await.expect("request to web application failed");
        dbg!(&response);
        $(
            $crate::check_condition!(response, $cond, $value $(, $extra_value)?);
        )*
    };
}
