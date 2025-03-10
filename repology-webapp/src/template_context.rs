// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Result, anyhow};

use crate::endpoints::{Endpoint, Section};
use crate::static_files::STATIC_FILES;
use crate::url_for::UrlConstructor;

pub struct TemplateContext {
    pub endpoint: Endpoint,
    params: Vec<(String, String)>,
}

impl TemplateContext {
    pub fn new(
        endpoint: Endpoint,
        path_params: Vec<(String, String)>,
        query_params: Vec<(String, String)>,
    ) -> Self {
        Self {
            endpoint,
            params: path_params.into_iter().chain(query_params).collect(),
        }
    }

    pub fn new_without_params(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            params: Default::default(),
        }
    }

    pub fn url_for_static(&self, file_name: &str) -> Result<String> {
        let file = STATIC_FILES
            .by_orig_name(file_name)
            .ok_or_else(|| anyhow!("unknown static file \"{}\"", file_name))?;

        UrlConstructor::new(Endpoint::StaticFile.path())
            .with_field("file_name", &file.hashed_name)
            .construct()
    }

    pub fn url_for_unversioned_static(&self, file_name: &str) -> Result<String> {
        UrlConstructor::new(Endpoint::StaticFile.path())
            .with_field("file_name", file_name)
            .construct()
    }

    #[expect(dead_code)]
    pub fn external_url_for_static(&self, file_name: &str) -> Result<String> {
        Ok(crate::constants::REPOLOGY_HOSTNAME.to_string() + &self.url_for_static(file_name)?)
    }

    pub fn external_url_for_unversioned_static(&self, file_name: &str) -> Result<String> {
        Ok(crate::constants::REPOLOGY_HOSTNAME.to_string()
            + &self.url_for_unversioned_static(file_name)?)
    }

    pub fn url_for<'a>(&self, endpoint: Endpoint, fields: &[(&'a str, &'a str)]) -> Result<String> {
        UrlConstructor::new(endpoint.path())
            .with_fields(fields.iter().cloned())
            .construct()
    }

    pub fn external_url_for<'a>(
        &self,
        endpoint: Endpoint,
        fields: &[(&'a str, &'a str)],
    ) -> Result<String> {
        Ok(crate::constants::REPOLOGY_HOSTNAME.to_string() + &self.url_for(endpoint, fields)?)
    }

    pub fn url_for_self<'a>(&self, fields: &[(&'a str, &'a str)]) -> Result<String> {
        UrlConstructor::new(self.endpoint.path())
            .with_fields(self.params.iter().map(|(k, v)| (k.as_ref(), v.as_ref())))
            .with_fields(fields.iter().cloned())
            .construct()
    }

    pub fn external_url_for_self<'a>(&self, fields: &[(&'a str, &'a str)]) -> Result<String> {
        Ok(crate::constants::REPOLOGY_HOSTNAME.to_string() + &self.url_for_self(fields)?)
    }

    pub fn is_section(&self, section: Section) -> bool {
        self.endpoint.is_section(section)
    }

    pub fn is_endpoint(&self, endpoint: Endpoint) -> bool {
        self.endpoint == endpoint
    }

    pub fn needs_ipv6_notice(&self) -> bool {
        false
    }

    pub fn experimental_enabled(&self) -> bool {
        false
    }

    // TODO: hack before askama 12.2 with built-in deref filter is released
    pub fn deref<T: Copy>(&self, r: &T) -> T {
        *r
    }

    // XXX: convert to free functions
    pub fn format_percent<T>(&self, divident: &T, divisor: &T) -> String
    where
        T: num_traits::Num + num_traits::cast::AsPrimitive<f32>,
    {
        if *divisor == T::zero() {
            "-".into()
        } else {
            format!("{:.1}%", 100.0 * divident.as_() / divisor.as_())
        }
    }

    pub fn format_percent_longer<T>(&self, divident: &T, divisor: &T) -> String
    where
        T: num_traits::Num + num_traits::cast::AsPrimitive<f32>,
    {
        if *divisor == T::zero() {
            "-".into()
        } else {
            format!("{:.2}%", 100.0 * divident.as_() / divisor.as_())
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_format_percent() {
        let ctx = TemplateContext::new_without_params(crate::endpoints::Endpoint::Index);

        assert_eq!(ctx.format_percent(&1_usize, &2_usize), "50.0%".to_owned());
        assert_eq!(ctx.format_percent(&1_i32, &2_i32), "50.0%".to_owned());

        assert_eq!(
            ctx.format_percent_longer(&1_usize, &2_usize),
            "50.00%".to_owned()
        );
        assert_eq!(
            ctx.format_percent_longer(&1_i32, &2_i32),
            "50.00%".to_owned()
        );
    }
}
