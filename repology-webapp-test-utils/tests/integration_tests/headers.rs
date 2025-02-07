// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tokio::test]
async fn test_header_present() {
    let resp = perform_mock_request("").await;
    assert!(resp.header_present("content-type"));
    assert!(!resp.header_present("type-content"));
}

#[tokio::test]
async fn test_header_value() {
    let resp = perform_mock_request("").await;
    assert_eq!(
        resp.header_value("content-type"),
        Some(&http::header::HeaderValue::from_str("text/plain").unwrap())
    );
    assert_eq!(resp.header_value("nonexistent"), None);
}

#[tokio::test]
async fn test_header_value_str() {
    let resp = perform_mock_request("").await;
    assert_eq!(
        resp.header_value_str("content-type")
            .expect("header should be parsable"),
        Some("text/plain")
    );
    assert_eq!(
        resp.header_value_str("nonexistent")
            .expect("header should be parsable"),
        None
    );
}
