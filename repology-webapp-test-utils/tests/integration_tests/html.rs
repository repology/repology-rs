// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

use repology_webapp_test_utils::HtmlValidationFlags;

#[tokio::test]
async fn test_is_html_valid() {
    let resp = perform_mock_request(
        "<!DOCTYPE html><html><head><title>test</title></head><body></body></html>",
    )
    .await;
    assert!(resp.is_html_valid(HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[tokio::test]
async fn test_is_html_valid_warnings() {
    let resp = perform_mock_request("<html></html>").await;
    assert!(resp.is_html_valid(HtmlValidationFlags::default()));
    assert!(!resp.is_html_valid(HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[tokio::test]
async fn test_is_html_valid_empty_tags() {
    let resp = perform_mock_request(
        "<!DOCTYPE html><html><head><title>test</title></head><body><h1></h1></body></html>",
    )
    .await;
    assert!(!resp.is_html_valid(HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(resp.is_html_valid(
        HtmlValidationFlags::WARNINGS_ARE_FATAL | HtmlValidationFlags::ALLOW_EMPTY_TAGS
    ));
}
