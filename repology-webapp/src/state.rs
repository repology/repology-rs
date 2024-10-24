// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use sqlx::PgPool;

use crate::font::FontMeasurer;
use crate::repository_data::RepositoryDataCache;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub font_measurer: Arc<FontMeasurer>,
    pub repository_data_cache: RepositoryDataCache,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        font_measurer: FontMeasurer,
        repository_data_cache: RepositoryDataCache,
    ) -> AppState {
        Self {
            pool,
            font_measurer: Arc::new(font_measurer),
            repository_data_cache,
        }
    }
}
