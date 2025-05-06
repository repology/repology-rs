// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
mod tests;

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use async_trait::async_trait;
use tracing::error;

use crate::http_client::{HttpClient, HttpMethod, HttpRequest, HttpResponse};
use crate::status::HttpStatus;

struct FakeResolver {
    address: IpAddr,
}

impl FakeResolver {
    pub fn new(address: IpAddr) -> Self {
        Self { address }
    }
}

impl reqwest::dns::Resolve for FakeResolver {
    fn resolve(&self, _: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let addrs = std::iter::once(SocketAddr::new(self.address, 0));
        Box::pin(async move { Ok(Box::new(addrs) as Box<_>) })
    }
}

struct StatusChooser {
    status: HttpStatus,
}

impl StatusChooser {
    fn new() -> Self {
        Self {
            status: HttpStatus::UnknownError,
        }
    }

    fn push(&mut self, status: HttpStatus) {
        if status.precision() > self.status.precision() {
            self.status = status
        } else if status.precision() == self.status.precision() && status != self.status {
            error!(status = ?self.status, another_status = ?status, "conflicting statuses of same precision")
        }
    }

    fn get(&self) -> HttpStatus {
        self.status
    }
}

pub struct NativeHttpClient {
    user_agent: String,
}

impl NativeHttpClient {
    pub fn new(user_agent: String) -> Self {
        Self { user_agent }
    }
}

fn process_response(response: reqwest::Response) -> HttpResponse {
    let location = if let Some(location) = response.headers().get("location") {
        if let Ok(location) = location.to_str() {
            Some(location.to_string())
        } else {
            return HttpResponse {
                status: HttpStatus::InvalidUrl,
                location: None,
            };
        }
    } else {
        None
    };

    HttpResponse {
        status: HttpStatus::Http(response.status().as_u16()),
        location,
    }
}

fn extract_status_from_error_hierarchy(
    error: &(dyn std::error::Error + 'static),
    chooser: &mut StatusChooser,
) {
    if let Some(source) = error.source() {
        extract_status_from_error_hierarchy(source, chooser);
    }

    // first try precise error type matching
    if let Some(error) = error.downcast_ref::<std::io::Error>() {
        match error.kind() {
            std::io::ErrorKind::HostUnreachable => chooser.push(HttpStatus::HostUnreachable),
            std::io::ErrorKind::ConnectionRefused => chooser.push(HttpStatus::ConnectionRefused),
            _ => {}
        }
    }
    if let Some(error) = error.downcast_ref::<hyper::Error>() {
        if error.is_incomplete_message() {
            chooser.push(HttpStatus::ServerDisconnected);
        }
    }

    // then fallback to parsing error Display output
    const STRING_MATCHES: &[(&str, HttpStatus)] = &[
        ("SSL routines", HttpStatus::SslError),
        (
            "self-signed certificate",
            HttpStatus::SslCertificateSelfSigned,
        ),
        (
            "unable to get local issuer certificate",
            HttpStatus::SslCertificateIncompleteChain,
        ),
    ];

    let descr = error.to_string();
    for (substring, status) in STRING_MATCHES {
        if descr.contains(substring) {
            chooser.push(*status)
        }
    }
}

fn error_to_status(error: reqwest::Error) -> HttpStatus {
    if error.is_timeout() {
        return HttpStatus::Timeout;
    }

    let mut chooser = StatusChooser::new();

    extract_status_from_error_hierarchy(&error, &mut chooser);

    let status = chooser.get();

    if status == HttpStatus::UnknownError {
        error!(?error, "unhandled error type");
    }

    status
}

fn process_error(error: reqwest::Error) -> HttpResponse {
    HttpResponse {
        status: error_to_status(error),
        location: None,
    }
}

#[async_trait]
impl HttpClient for NativeHttpClient {
    async fn request(&self, request: HttpRequest) -> HttpResponse {
        let client = reqwest::ClientBuilder::new()
            .user_agent(&self.user_agent)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(request.timeout)
            .dns_resolver(Arc::new(FakeResolver::new(request.address)))
            .build()
            .expect("expected to always be able to build reqwest client");

        match client
            .request(
                match request.method {
                    HttpMethod::Head => reqwest::Method::HEAD,
                    HttpMethod::Get => reqwest::Method::GET,
                },
                request.url,
            )
            .send()
            .await
        {
            Ok(response) => process_response(response),
            Err(error) => process_error(error),
        }
    }
}
