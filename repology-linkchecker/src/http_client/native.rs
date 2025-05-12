// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
mod tests;

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use async_trait::async_trait;

use crate::errors::extract_status;
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

pub struct NativeHttpClient {
    user_agent: String,
}

impl NativeHttpClient {
    pub fn new(user_agent: String) -> Self {
        Self { user_agent }
    }
}

#[async_trait]
impl HttpClient for NativeHttpClient {
    async fn request(&self, request: HttpRequest) -> HttpResponse {
        let client = reqwest::ClientBuilder::new()
            .user_agent(&self.user_agent)
            .redirect(reqwest::redirect::Policy::none())
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
            .timeout(request.timeout)
            .send()
            .await
        {
            Ok(response) => {
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
            Err(error) => HttpResponse {
                status: extract_status(&error, error.url().map(url::Url::as_str).unwrap_or("???")),
                location: None,
            },
        }
    }
}
