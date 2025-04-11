// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;
use std::sync::Arc;
use tracing::{Instrument as _, error, info, info_span};

use crate::state::AppState;

pub fn start_repository_data_cache_task(state: Arc<AppState>) {
    let state = Arc::downgrade(&state);
    let task = async move {
        loop {
            tokio::time::sleep(crate::constants::REPOSITORY_CACHE_REFRESH_PERIOD).await;

            let Some(state) = state.upgrade() else { break };

            state
                .repository_data_cache
                .update()
                .await
                .unwrap_or_else(|e| error!("repository data cache update failed: {:?}", e));
        }
    };
    tokio::task::spawn(
        task.instrument(info_span!(parent: None, "repository data cache background task")),
    );
}

pub fn start_important_projects_cache_task(state: Arc<AppState>, pool: PgPool) {
    let state = Arc::downgrade(&state);
    let task = async move {
        loop {
            tokio::time::sleep(crate::constants::IMPORTANT_PROJECTS_CACHE_REFRESH_PERIOD).await;

            let Some(state) = state.upgrade() else { break };

            let important_projects_cache = match crate::views::get_important_projects(&pool).await {
                Ok(important_projects_cache) => Arc::new(important_projects_cache),
                Err(e) => {
                    error!("important projects cache update failed: {:?}", e);
                    continue;
                }
            };
            let num_entries = important_projects_cache.len();
            if let Err(e) = state.important_projects_cache.set(important_projects_cache) {
                error!("important projects cache update failed: {:?}", e);
                continue;
            }
            info!("updated important projects cache, {} entries", num_entries);
        }
    };
    tokio::task::spawn(
        task.instrument(info_span!(parent: None, "important projects cache background task")),
    );
}
