// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![coverage(off)]

use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use axum::http::{HeaderValue, StatusCode, header};
use axum::routing::get;

use repology_common::LinkStatus;

use crate::http_client::{HttpClient, HttpMethod, HttpRequest};

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
        )
        .route(
            "/timeout",
            get(async || {
                tokio::time::sleep(Duration::from_secs(60)).await;
                (StatusCode::OK, String::new())
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
async fn test_request_200() {
    let http_client = HttpClient::new("repology/linkchecker".to_string());
    let (ipv4_addr, ipv6_addr) = run_test_server().await;

    let response = http_client
        .request(HttpRequest {
            url: format!("http://example.com:{}/200", ipv4_addr.port()),
            method: HttpMethod::Head,
            address: ipv4_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, LinkStatus::Http(200));
    assert_eq!(response.location, None);

    let response = http_client
        .request(HttpRequest {
            url: format!("http://example.com:{}/200", ipv6_addr.port()),
            method: HttpMethod::Head,
            address: ipv6_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, LinkStatus::Http(200));
    assert_eq!(response.location, None);
}

#[tokio::test]
async fn test_request_404() {
    let http_client = HttpClient::new("repology/linkchecker".to_string());
    let (ipv4_addr, ipv6_addr) = run_test_server().await;

    let response = http_client
        .request(HttpRequest {
            url: format!("http://example.com:{}/404", ipv4_addr.port()),
            method: HttpMethod::Head,
            address: ipv4_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, LinkStatus::Http(404));
    assert_eq!(response.location, None);

    let response = http_client
        .request(HttpRequest {
            url: format!("http://example.com:{}/404", ipv6_addr.port()),
            method: HttpMethod::Head,
            address: ipv6_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, LinkStatus::Http(404));
    assert_eq!(response.location, None);
}

#[tokio::test]
async fn test_request_redirect() {
    let http_client = HttpClient::new("repology/linkchecker".to_string());
    let (ipv4_addr, ipv6_addr) = run_test_server().await;

    let response = http_client
        .request(HttpRequest {
            url: format!("http://example.com:{}/308", ipv4_addr.port()),
            method: HttpMethod::Head,
            address: ipv4_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, LinkStatus::Http(308));
    assert_eq!(response.location, Some("/".to_string()));

    let response = http_client
        .request(HttpRequest {
            url: format!("http://example.com:{}/308", ipv6_addr.port()),
            method: HttpMethod::Head,
            address: ipv6_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, LinkStatus::Http(308));
    assert_eq!(response.location, Some("/".to_string()));
}

#[tokio::test]
async fn test_request_timeout() {
    let requester = HttpClient::new("repology/linkchecker".to_string());
    let (ipv4_addr, ipv6_addr) = run_test_server().await;

    let response = requester
        .request(HttpRequest {
            url: format!("http://example.com:{}/timeout", ipv4_addr.port()),
            method: HttpMethod::Head,
            address: ipv4_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, LinkStatus::Timeout);
    assert_eq!(response.location, None);

    let response = requester
        .request(HttpRequest {
            url: format!("http://example.com:{}/timeout", ipv6_addr.port()),
            method: HttpMethod::Head,
            address: ipv6_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, LinkStatus::Timeout);
    assert_eq!(response.location, None);
}

#[tokio::test]
async fn test_request_ssl_error() {
    let http_client = HttpClient::new("repology/linkchecker".to_string());
    let (ipv4_addr, ipv6_addr) = run_test_server().await;

    let response = http_client
        .request(HttpRequest {
            url: format!("https://example.com:{}/200", ipv4_addr.port()),
            method: HttpMethod::Head,
            address: ipv4_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, LinkStatus::SslError);
    assert_eq!(response.location, None);

    let response = http_client
        .request(HttpRequest {
            url: format!("https://example.com:{}/200", ipv6_addr.port()),
            method: HttpMethod::Head,
            address: ipv6_addr.ip(),
            timeout: Duration::from_secs(1),
        })
        .await;
    assert_eq!(response.status, LinkStatus::SslError);
    assert_eq!(response.location, None);
}
