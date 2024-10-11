// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{anyhow, Error};
use askama::Template;
use axum::extract::{Path, Query};
use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;

use crate::endpoints::{Endpoint, Section};
use crate::result::EndpointResult;
use crate::static_files::StaticFiles;
use crate::url_for::UrlConstructor;

#[derive(Template)]
#[template(path = "news.html")]
struct TemplateParams {
    endpoint: Endpoint,
}

impl TemplateParams {
    pub fn url_for_static(&self, file_name: &str) -> Result<String, Error> {
        let name_map = StaticFiles::new().name_to_hashed_name_map();

        let hashed_file_name = name_map
            .get(file_name)
            .ok_or_else(|| anyhow!("unknown static file \"{}\"", file_name))?
            .to_string();

        Ok(UrlConstructor::new(Endpoint::StaticFile.path())
            .with_field("file_name", &hashed_file_name)
            .construct()?)
    }

    pub fn url_for<'a>(
        &self,
        endpoint: Endpoint,
        fields: &[(&'a str, &'a str)],
    ) -> Result<String, Error> {
        Ok(UrlConstructor::new(endpoint.path())
            .with_fields(fields.iter().cloned())
            .construct()?)
    }

    pub fn is_section(&self, section: Section) -> bool {
        self.endpoint.is_section(section)
    }

    pub fn needs_ipv6_notice(&self) -> bool {
        false
    }

    pub fn admin(&self) -> bool {
        false
    }

    pub fn experimental_enabled(&self) -> bool {
        false
    }

    pub fn is_repology_rs(&self) -> bool {
        true
    }
}

#[tracing::instrument()]
pub async fn news(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
) -> EndpointResult {
    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            endpoint: Endpoint::News,
        }
        .render()?,
    )
        .into_response())
}
