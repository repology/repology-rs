// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
mod tests;

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use tracing::warn;

use repology_common::LinkStatus;

use crate::errors::extract_status;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Head,
    Get,
}

impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Head => "HEAD",
            Self::Get => "GET",
        }
    }
}

#[derive(Clone)]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub address: IpAddr,
    pub timeout: Duration,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: LinkStatus,
    pub location: Option<String>,
}

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

pub struct HttpClient {
    user_agent: String,
}

impl HttpClient {
    pub fn new(user_agent: String) -> Self {
        Self { user_agent }
    }

    pub async fn request(&self, request: HttpRequest) -> HttpResponse {
        let client = reqwest::ClientBuilder::new()
            .user_agent(&self.user_agent)
            .redirect(reqwest::redirect::Policy::none())
            .dns_resolver(Arc::new(FakeResolver::new(request.address)))
            .retry(reqwest::retry::never())
            .build()
            .expect("expected to always be able to build reqwest client");

        match client
            .request(
                match request.method {
                    HttpMethod::Head => reqwest::Method::HEAD,
                    HttpMethod::Get => reqwest::Method::GET,
                },
                &request.url,
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
                        warn!(url = request.url, ?location, "cannot parse location header");
                        return HttpResponse {
                            status: LinkStatus::InvalidUrl,
                            location: None,
                        };
                    }
                } else {
                    None
                };

                HttpResponse {
                    status: LinkStatus::Http(response.status().as_u16()),
                    location,
                }
            }
            Err(error) => HttpResponse {
                status: extract_status(&error, &request.url),
                location: None,
            },
        }
    }
}
