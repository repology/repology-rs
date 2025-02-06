// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use bitflags::bitflags;
use sqlx::PgPool;
use std::collections::HashMap;
use thiserror::Error;
use tower_service::Service;

use repology_webapp::{config::AppConfig, create_app};

#[derive(Default)]
pub struct Request {
    pool: Option<PgPool>,
    uri: Option<String>,
    form: Option<String>,
    headers: HashMap<String, String>,
    config: AppConfig,
    xml_namespaces: HashMap<String, String>,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ResponseError {
    #[error("reponse must be valid utf-8 document")]
    Utf8Error(String),

    #[error("reponse must be valid XML document")]
    XmlError(String),

    #[error("reponse must be valid JSON document")]
    JsonError(String),

    #[error("reponse contains invalid header")]
    HeaderError,

    #[error("reponse contains invalid header")]
    XpathError(String),
}

#[derive(Debug, Default)]
pub struct Response {
    status: http::StatusCode,
    headers: http::header::HeaderMap,
    body: bytes::Bytes,

    xml_namespaces: HashMap<String, String>,
    text: std::cell::OnceCell<Result<String, ResponseError>>,
    xml: std::cell::OnceCell<Result<sxd_document::Package, ResponseError>>,
}

impl Clone for Response {
    fn clone(&self) -> Self {
        Response {
            status: self.status,
            headers: self.headers.clone(),
            body: self.body.clone(),
            xml_namespaces: self.xml_namespaces.clone(),
            text: Default::default(),
            xml: Default::default(),
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct HtmlValidationFlags: u32 {
        const WARNINGS_ARE_FATAL = 1 << 0;
        const ALLOW_EMPTY_TAGS = 1 << 1;
    }
}

impl Request {
    pub fn new(pool: PgPool, uri: &str) -> Self {
        Self {
            pool: Some(pool),
            uri: Some(uri.to_owned()),
            ..Default::default()
        }
    }

    // builder interface
    pub fn with_uri(mut self, uri: &str) -> Self {
        self.uri = Some(uri.to_owned());
        self
    }

    pub fn with_form<T: serde::Serialize>(mut self, data: T) -> Self {
        self.form = Some(serde_urlencoded::to_string(data).expect("failed to serialize form data"));
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_spam_keyword(mut self, keyword: &str) -> Self {
        self.config.spam_keywords.push(keyword.to_string());
        self
    }

    pub fn with_spam_network(mut self, network: &ip_network::IpNetwork) -> Self {
        self.config.spam_networks.push(network.clone());
        self
    }

    pub fn with_reports_disabled(mut self, project_name: &str) -> Self {
        self.config
            .disabled_reports
            .insert(project_name.to_string());
        self
    }

    pub fn with_xml_namespace(mut self, key: &str, value: &str) -> Self {
        self.xml_namespaces
            .insert(key.to_string(), value.to_string());
        self
    }

    // performing the request
    pub async fn perform(self) -> Response {
        let pool = self.pool.clone().expect("pool must be set");
        let config = self.config.clone();

        self.perform_with(
            create_app(pool, config)
                .await
                .expect("failed to create application"),
        )
        .await
    }

    pub async fn perform_with(self, mut router: axum::Router) -> Response {
        let mut request =
            axum::http::Request::builder().uri(self.uri.as_deref().expect("uri must be set"));
        for (k, v) in &self.headers {
            request = request.header(k, v);
        }

        let request = if let Some(form) = &self.form {
            request = request.method("POST");
            request = request.header("content-type", "application/x-www-form-urlencoded");
            request
                .body(form.clone())
                .expect("failed to set request body")
        } else {
            request = request.method("GET");
            request
                .body("".to_owned())
                .expect("failed to set request body")
        };

        let response = router
            .call(request)
            .await
            .expect("failed to perform application request");
        let status = response.status();
        let headers = response.headers().clone();
        let body = axum::body::to_bytes(response.into_body(), 1000000)
            .await
            .expect("failed to extract response body");

        let response = Response {
            status,
            headers,
            body,
            xml_namespaces: self.xml_namespaces,
            ..Default::default()
        };
        dbg!(&response);
        response
    }
}

impl Response {
    pub fn with_xml_namespace(mut self, key: &str, value: &str) -> Self {
        self.xml_namespaces
            .insert(key.to_string(), value.to_string());
        self
    }

    // getters
    pub fn text(&self) -> Result<&str, ResponseError> {
        self.text
            .get_or_init(|| {
                std::str::from_utf8(&self.body)
                    .map(|s| s.to_owned())
                    .map_err(|e| ResponseError::Utf8Error(format!("{}", e)))
            })
            .as_deref()
            .map_err(|e| e.clone())
    }

    pub fn xml(&self) -> Result<sxd_document::dom::Document, ResponseError> {
        self.xml
            .get_or_init(|| {
                sxd_document::parser::parse(self.text()?)
                    .map_err(|e| ResponseError::XmlError(format!("{}", e)))
            })
            .as_ref()
            .map(|d| d.as_document())
            .map_err(|e| e.clone())
    }

    pub fn status(&self) -> http::StatusCode {
        self.status
    }

    pub fn header_present(&self, key: &str) -> bool {
        self.headers.contains_key(key)
    }

    pub fn header_value(&self, key: &str) -> Option<&http::header::HeaderValue> {
        self.headers.get(key)
    }

    pub fn header_value_str(&self, key: &str) -> Result<Option<&str>, ResponseError> {
        self.headers
            .get(key)
            .map(|h| h.to_str().map_err(|_| ResponseError::HeaderError))
            .transpose()
    }

    pub fn body_cityhash64(&self) -> u64 {
        cityhasher::hash::<u64>(&self.body)
    }

    pub fn body_length(&self) -> usize {
        self.body.len()
    }

    pub fn is_html_valid(&self, flags: HtmlValidationFlags) -> bool {
        let Ok(text) = self.text() else {
            return false;
        };
        let mut validation_result = crate::tidy::validate_html(text);
        dbg!(&validation_result);

        if flags.contains(HtmlValidationFlags::ALLOW_EMPTY_TAGS) {
            validation_result
                .output
                .retain(|s| !s.contains("Warning: trimming empty <"));
        }

        match validation_result.status {
            crate::tidy::ValidationStatus::Ok | crate::tidy::ValidationStatus::WithWarnings => {
                validation_result.output.is_empty()
                    || !flags.contains(HtmlValidationFlags::WARNINGS_ARE_FATAL)
            }
            _ => false,
        }
    }

    pub fn json(&self) -> Result<serde_json::Value, ResponseError> {
        serde_json::from_str(self.text()?).map_err(|e| ResponseError::JsonError(format!("{}", e)))
    }

    #[track_caller]
    pub fn xpath(&self, xpath: &str) -> Result<sxd_xpath::Value, ResponseError> {
        let factory = sxd_xpath::Factory::new();

        // build() returns Result<Option>, not sure what Ok(None) really means here,
        // treating it the same way as Err
        let xpath = factory
            .build(xpath)
            .expect("invalid XPath")
            .expect("invalid XPath");

        let mut context = sxd_xpath::Context::new();

        for ns in &self.xml_namespaces {
            context.set_namespace(ns.0, ns.1);
        }

        xpath
            .evaluate(&context, self.xml()?.root())
            .map_err(|e| ResponseError::XpathError(format!("{}", e)))
    }

    // snapshot
    pub fn as_snapshot(&self) -> Result<String, ResponseError> {
        let mut snapshot: String = format!("Status: {}\n", self.status.as_u16());
        for (k, v) in &self.headers {
            snapshot += &format!("Header: {}: {}\n", k, v.to_str().unwrap());
        }
        snapshot += "---\n";
        snapshot += self.text()?;
        Ok(snapshot)
    }
}
