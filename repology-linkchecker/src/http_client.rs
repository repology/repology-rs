// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod native;
pub mod python;

use std::net::IpAddr;
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;

use repology_common::LinkStatus;

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

#[derive(Deserialize, Debug)]
pub struct HttpResponse {
    pub status: LinkStatus,
    pub location: Option<String>,
    pub is_cloudflare: bool,
    pub is_iis: bool,
}

#[async_trait]
pub trait HttpClient {
    async fn request(&self, request: HttpRequest) -> HttpResponse;
}
