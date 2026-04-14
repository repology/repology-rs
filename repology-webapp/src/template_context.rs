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
}
