// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tokio::test]
async fn test_xpath_str() {
    let resp = perform_mock_request("<xml><a>foo</a></xml>").await;
    assert_eq!(resp.xpath("string(/xml/a)").unwrap(), "foo");
    assert_eq!(resp.xpath("string(/xml/b)").unwrap(), "");
}

#[tokio::test]
async fn test_xpath_num() {
    let resp = perform_mock_request("<xml><a>1</a></xml>").await;
    assert_eq!(resp.xpath("number(/xml/a)").unwrap(), 1_f64);
    assert_eq!(resp.xpath("count(/xml/a)").unwrap(), 1_f64);
    assert_eq!(resp.xpath("count(/xml/b)").unwrap(), 0_f64);
}

#[tokio::test]
async fn test_xpath_namespace() {
    use repology_webapp_test_utils::Request;

    let resp = Request::default()
        .with_uri("/")
        .with_xml_namespace("svg", "http://www.w3.org/2000/svg")
        .perform_with(create_mock_router(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><g></g></svg>"#,
        ))
        .await;

    assert_eq!(resp.xpath("count(/svg:svg/svg:g)").unwrap(), 1_f64);
}

#[tokio::test]
#[should_panic]
async fn test_xpath_invalid() {
    let resp = perform_mock_request("<xml><a>foo</a></xml>").await;
    _ = resp.xpath("some invalid xpath");
}
