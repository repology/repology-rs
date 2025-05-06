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

fn extract_status_from_io_error(
    error: &std::io::Error,
    chooser: &mut StatusChooser,
    url: Option<&str>,
) {
    use std::io::ErrorKind::*;
    match error.kind() {
        HostUnreachable => chooser.push(HttpStatus::HostUnreachable),
        ConnectionRefused => chooser.push(HttpStatus::ConnectionRefused),
        UnexpectedEof => chooser.push(HttpStatus::ConnectionResetByPeer),
        _ => {}
    }
    if let Some(inner) = error.get_ref() {
        extract_status_from_error_hierarchy(inner, chooser, url)
    }
}

fn extract_status_from_hyper_error(
    error: &hyper::Error,
    chooser: &mut StatusChooser,
    _url: Option<&str>,
) {
    if error.is_incomplete_message() {
        chooser.push(HttpStatus::ServerDisconnected);
    }
}

fn extract_status_from_rustls_error(
    error: &rustls::Error,
    chooser: &mut StatusChooser,
    url: Option<&str>,
) {
    use rustls::Error::*;
    match error {
        InvalidCertificate(certificate_error) => {
            extract_status_from_rustls_certificate_error(certificate_error, chooser, url)
        }
        AlertReceived(alert_description) => {
            extract_status_from_rustls_alert_description(alert_description, chooser, url)
        }
        Other(other_error) => extract_status_from_rustls_other_error(&other_error, chooser, url),
        _ => {
            chooser.push(HttpStatus::SslError);
            error!(?error, ?url, "unhandled rustls::Error variant");
        }
    }
}

fn extract_status_from_rustls_alert_description(
    error: &rustls::AlertDescription,
    chooser: &mut StatusChooser,
    url: Option<&str>,
) {
    use rustls::AlertDescription::*;
    match error {
        HandshakeFailure => chooser.push(HttpStatus::SslError), // XXX: we need more specific error code for it
        _ => {
            chooser.push(HttpStatus::SslError);
            error!(?error, ?url, "unhandled rustls::AlertDescription variant");
        }
    }
}

fn extract_status_from_rustls_certificate_error(
    error: &rustls::CertificateError,
    chooser: &mut StatusChooser,
    url: Option<&str>,
) {
    use rustls::CertificateError::*;
    match error {
        Expired => chooser.push(HttpStatus::SslCertificateHasExpired),
        UnknownIssuer => chooser.push(HttpStatus::SslCertificateIncompleteChain),
        Other(other_error) => extract_status_from_rustls_other_error(&other_error, chooser, url),
        _ => {
            chooser.push(HttpStatus::SslError);
            error!(?error, ?url, "unhandled rustls::CertificateError variant");
        }
    }
}

fn extract_status_from_webpki_error(
    error: &webpki::Error,
    chooser: &mut StatusChooser,
    url: Option<&str>,
) {
    use webpki::Error::*;
    match error {
        CaUsedAsEndEntity => chooser.push(HttpStatus::SslCertificateSelfSigned),
        _ => {
            chooser.push(HttpStatus::SslError);
            error!(?error, ?url, "unhandled webpki::Error variant");
        }
    }
}

fn extract_status_from_rustls_other_error(
    error: &rustls::OtherError,
    chooser: &mut StatusChooser,
    url: Option<&str>,
) {
    extract_status_from_error_hierarchy(error.0.as_ref(), chooser, url);
}

fn extract_status_from_error_hierarchy(
    error: &(dyn std::error::Error + 'static),
    chooser: &mut StatusChooser,
    url: Option<&str>,
) {
    if let Some(source) = error.source() {
        extract_status_from_error_hierarchy(source, chooser, url)
    }

    error
        .downcast_ref::<std::io::Error>()
        .inspect(|error| extract_status_from_io_error(error, chooser, url));
    error
        .downcast_ref::<hyper::Error>()
        .inspect(|error| extract_status_from_hyper_error(error, chooser, url));
    error
        .downcast_ref::<rustls::Error>()
        .inspect(|error| extract_status_from_rustls_error(error, chooser, url));
    error
        .downcast_ref::<webpki::Error>()
        .inspect(|error| extract_status_from_webpki_error(error, chooser, url));
}

fn error_to_status(error: reqwest::Error) -> HttpStatus {
    if error.is_timeout() {
        return HttpStatus::Timeout;
    }

    let mut chooser = StatusChooser::new();

    extract_status_from_error_hierarchy(
        &error,
        &mut chooser,
        error.url().map(reqwest::Url::as_str),
    );

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
            .use_rustls_tls()
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
