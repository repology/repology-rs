// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod __private {
    pub use crate::HtmlValidationFlags;

    pub use axum;
    pub use mime;
    pub use pretty_assertions;
    pub use regex;
}

#[macro_export]
macro_rules! check_condition {
    ($resp:ident, status, $status:literal) => {
        assert_eq!($resp.status(), $status);
    };
    ($resp:ident, status, $status:ident) => {
        assert_eq!(
            $resp.status(),
            $crate::__private::axum::http::StatusCode::$status
        );
    };
    ($resp:ident, content_type, $content_type:literal) => {
        assert_eq!(
            $resp
                .header_value("content-type")
                .and_then(|v| v.to_str().ok()),
            Some($content_type)
        );
    };
    ($resp:ident, content_type, $content_type:ident) => {
        assert_eq!(
            $resp
                .header_value("content-type")
                .and_then(|v| v.to_str().ok()),
            Some($crate::__private::mime::$content_type.as_ref().into())
        );
    };
    ($resp:ident, contains, $text:literal) => {
        assert!(
            $resp.text().unwrap().contains($text),
            "contains condition with text \"{}\" has not matched",
            $text
        );
    };
    ($resp:ident, contains_not, $text:literal) => {
        assert!(
            !$resp.text().unwrap().contains($text),
            "contains_not condition with text \"{}\" has unexpectedly matched",
            $text
        );
    };
    ($resp:ident, body_length, $length:literal) => {
        assert_eq!($resp.body_length(), $length);
    };
    ($resp:ident, body_cityhash64, $hash:literal) => {
        assert_eq!($resp.body_cityhash64(), $hash);
    };
    ($resp:ident, json, $json:literal) => {
        use std::str::FromStr as _;
        $crate::__private::pretty_assertions::assert_eq!(
            $resp.json().unwrap(),
            serde_json::Value::from_str($json).unwrap()
        );
    };
    ($resp:ident, xpath, $xpath:literal, $value:literal) => {{
        assert_eq!($resp.xpath($xpath).unwrap(), $value);
    }};
    ($resp:ident, svg_xpath, $xpath:literal, $value:literal) => {{
        assert_eq!(
            $resp
                .clone()
                .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
                .xpath($xpath)
                .unwrap(),
            $value
        );
    }};
    ($resp:ident, matches, $regexp:literal) => {
        assert!(
            $crate::__private::regex::Regex::new($regexp)
                .expect("failed to parse regex")
                .is_match($resp.text().unwrap()),
            "matches condition with regexp \"{}\" has not matched",
            $regexp
        );
    };
    ($resp:ident, matches_not, $regexp:literal) => {
        assert!(
            !$crate::__private::regex::Regex::new($regexp)
                .expect("failed to parse regex")
                .is_match($resp.text().unwrap()),
            "matches_not condition with regexp \"{}\" has unexpectedly matched",
            $regexp
        );
    };
    ($resp:ident, line_matches, $regexp:literal) => {
        let re = $crate::__private::regex::Regex::new($regexp).expect("failed to parse regex");
        assert!(
            $resp.text().unwrap().lines().any(|line| re.is_match(line)),
            "line_matches condition with regexp \"{}\" has not matched",
            $regexp
        );
    };
    ($resp:ident, line_matches_not, $regexp:literal) => {
        let re = $crate::__private::regex::Regex::new($regexp).expect("failed to parse regex");
        assert!(
            !$resp.text().unwrap().lines().any(|line| re.is_match(line)),
            "line_matches_not condition with regexp \"{}\" has unexpectedly matched",
            $regexp
        );
    };
    ($resp:ident, html_ok, $level:literal) => {{
        use repology_webapp_test_utils::HtmlValidationFlags;
        let mut flags = HtmlValidationFlags::default();

        for param in $level.split(',') {
            match param {
                "warnings_fatal" => flags |= HtmlValidationFlags::WARNINGS_ARE_FATAL,
                "allow_empty_tags" => flags |= HtmlValidationFlags::ALLOW_EMPTY_TAGS,
                "" => {}
                other => {
                    panic!("unknown html_ok flag {}", other);
                }
            }
        }

        assert!($resp.is_html_valid(flags));
    }};
    ($resp:ident, header_present, $header:literal) => {
        assert!($resp.header_present($header));
    };
    ($resp:ident, header_value, $header:literal, $value:literal) => {
        let header = $resp.header_value($header);
        assert!(header.is_some());
        assert_eq!(header.unwrap(), $value);
    };
    ($resp:ident, header_value_contains, $header:literal, $needle:literal) => {
        let header = $resp.header_value($header);
        assert!(header.is_some_and(|header| { header.to_str().unwrap().contains($needle) }));
    };
    ($resp:ident, header_value_contains_not, $header:literal, $needle:literal) => {
        let header = $resp.header_value($header);
        assert!(header.is_some_and(|header| { !header.to_str().unwrap().contains($needle) }));
    };
}

#[macro_export]
macro_rules! check_response {
    (
        $pool:ident,
        form $form:expr,
        $(add_header $header_name:literal $header_value:literal, )*
        $uri:literal
        $(, $cond:tt $value:tt $($extra_value:literal)?)*
        $(,)?
    ) => {
        let request = $crate::Request::new(
            $pool.clone(),
            $uri
        )
            .with_form($form)
            $(
            .with_header($header_name, $header_value)
            )*
        ;
        let response = request.perform().await;
        dbg!(&response);
        $(
            $crate::check_condition!(response, $cond, $value $(, $extra_value)?);
        )*
    };
    (
        $pool:ident,
        $(add_header $header_name:literal $header_value:literal, )*
        $uri:literal
        $(, $cond:tt $value:tt $($extra_value:literal)?)*
        $(,)?
    ) => {
        let request = $crate::Request::new($pool.clone(), $uri)
            $(
            .with_header($header_name, $header_value)
            )*
        ;
        let response = request.perform().await;
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
        let request = $crate::Request::new($pool.clone(), $uri);
        let response = request.perform().await;
        dbg!(&response);
        $(
            $crate::check_condition!(response, $cond, $value $(, $extra_value)?);
        )*
    };
}
