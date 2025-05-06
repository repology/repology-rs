// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![coverage(off)]

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use axum::Router;
use axum::http::{HeaderValue, StatusCode, header};
use axum::routing::get;

use crate::http_client::native::NativeHttpClient;
use crate::http_client::{HttpClient, HttpMethod, HttpRequest};
use crate::status::HttpStatus;

use serial_test::serial;

async fn run_test_server() -> (SocketAddr, SocketAddr) {
    let app = Router::new()
        .route("/200", get(async || (StatusCode::OK, String::new())))
        .route("/404", get(async || (StatusCode::NOT_FOUND, String::new())))
        .route(
            "/308",
            get(async || {
                (
                    StatusCode::PERMANENT_REDIRECT,
                    [(header::LOCATION, HeaderValue::from_static("/"))],
                    String::new(),
                )
            }),
        );

    (
        {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            tokio::task::spawn(axum::serve(listener, app.clone()).into_future());
            addr
        },
        {
            let listener = tokio::net::TcpListener::bind("[::1]:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            tokio::task::spawn(axum::serve(listener, app).into_future());
            addr
        },
    )
}

#[tokio::test]
#[serial(updater)]
async fn test_request_200() {
    let http_client = NativeHttpClient::new("repology/linkchecker".to_string());
    let (ipv4_addr, ipv6_addr) = run_test_server().await;

    let response = http_client
        .request(HttpRequest {
            url: format!("http://example.com:{}/200", ipv4_addr.port()),
            method: HttpMethod::Head,
            address: ipv4_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, HttpStatus::Http(200));
    assert_eq!(response.location, None);

    let response = http_client
        .request(HttpRequest {
            url: format!("http://example.com:{}/200", ipv6_addr.port()),
            method: HttpMethod::Head,
            address: ipv6_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, HttpStatus::Http(200));
    assert_eq!(response.location, None);
}

#[tokio::test]
#[serial(updater)]
async fn test_request_404() {
    let http_client = NativeHttpClient::new("repology/linkchecker".to_string());
    let (ipv4_addr, ipv6_addr) = run_test_server().await;

    let response = http_client
        .request(HttpRequest {
            url: format!("http://example.com:{}/404", ipv4_addr.port()),
            method: HttpMethod::Head,
            address: ipv4_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, HttpStatus::Http(404));
    assert_eq!(response.location, None);

    let response = http_client
        .request(HttpRequest {
            url: format!("http://example.com:{}/404", ipv6_addr.port()),
            method: HttpMethod::Head,
            address: ipv6_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, HttpStatus::Http(404));
    assert_eq!(response.location, None);
}

#[tokio::test]
#[serial(updater)]
async fn test_request_redirect() {
    let http_client = NativeHttpClient::new("repology/linkchecker".to_string());
    let (ipv4_addr, ipv6_addr) = run_test_server().await;

    let response = http_client
        .request(HttpRequest {
            url: format!("http://example.com:{}/308", ipv4_addr.port()),
            method: HttpMethod::Head,
            address: ipv4_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, HttpStatus::Http(308));
    assert_eq!(response.location, Some("/".to_string()));

    let response = http_client
        .request(HttpRequest {
            url: format!("http://example.com:{}/308", ipv6_addr.port()),
            method: HttpMethod::Head,
            address: ipv6_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, HttpStatus::Http(308));
    assert_eq!(response.location, Some("/".to_string()));
}

#[tokio::test]
#[serial(updater)]
async fn test_request_cannot_connect() {
    let http_client = NativeHttpClient::new("repology/linkchecker".to_string());

    // we use explicitly unreacheable addresses here, but statuses this
    // leads to may vary
    let ipv4_addr: IpAddr = "192.0.2.0".parse().unwrap();
    let ipv6_addr: IpAddr = "100::".parse().unwrap();
    let expected_statuses = [HttpStatus::Timeout, HttpStatus::NetworkUnreachable];

    let response = http_client
        .request(HttpRequest {
            url: "http://example.com/200".to_string(),
            method: HttpMethod::Head,
            address: ipv4_addr,
            timeout: Duration::from_secs(1),
        })
        .await;
    assert!(
        expected_statuses.contains(&response.status),
        "unexpected status {:?}",
        response.status
    );
    assert_eq!(response.location, None);

    let response = http_client
        .request(HttpRequest {
            url: "http://example.com/200".to_string(),
            method: HttpMethod::Head,
            address: ipv6_addr,
            timeout: Duration::from_secs(1),
        })
        .await;
    assert!(
        expected_statuses.contains(&response.status),
        "unexpected status {:?}",
        response.status
    );
    assert_eq!(response.location, None);
}
