// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Arc, Mutex};

use sqlx::PgPool;

use crate::config::AppConfig;
use crate::font::FontMeasurer;
use crate::repository_data::RepositoriesDataCache;

pub struct AppState {
    pub pool: PgPool,
    pub font_measurer: FontMeasurer,
    pub repository_data_cache: RepositoriesDataCache,
    pub config: AppConfig,
    pub important_projects_cache: Mutex<Arc<Vec<crate::views::ProjectListItem>>>,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        font_measurer: FontMeasurer,
        repository_data_cache: RepositoriesDataCache,
        config: AppConfig,
        important_projects_cache: Vec<crate::views::ProjectListItem>,
    ) -> Self {
        Self {
            pool,
            font_measurer,
            repository_data_cache,
            config,
            important_projects_cache: Mutex::new(Arc::new(important_projects_cache)),
        }
    }
}
