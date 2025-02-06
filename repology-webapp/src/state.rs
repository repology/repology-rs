// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use crate::config::AppConfig;
use crate::font::FontMeasurer;
use crate::repository_data::RepositoriesDataCache;

pub struct AppState {
    pub pool: PgPool,
    pub font_measurer: FontMeasurer,
    pub repository_data_cache: RepositoriesDataCache,
    pub config: AppConfig,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        font_measurer: FontMeasurer,
        repository_data_cache: RepositoriesDataCache,
        config: AppConfig,
    ) -> Self {
        Self {
            pool,
            font_measurer,
            repository_data_cache,
            config,
        }
    }
}
