// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod python;

use std::net::IpAddr;
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;

use crate::status::HttpStatus;

#[derive(Clone, Copy)]
pub enum HttpMethod {
    Head,
    Get,
}

pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub address: IpAddr,
    pub timeout: Duration,
}

#[derive(Deserialize, Debug)]
pub struct HttpResponse {
    pub status: HttpStatus,
    pub location: Option<String>,
}

#[async_trait]
pub trait HttpClient {
    async fn request(&self, request: HttpRequest) -> HttpResponse;
}
