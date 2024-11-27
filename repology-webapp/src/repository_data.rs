// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;

use anyhow::Result;
use indoc::indoc;
use metrics::gauge;
use serde::Deserialize;
use sqlx::{FromRow, PgPool};
use strum_macros::EnumString;
use tracing::info;

#[derive(Clone, Debug, PartialEq, EnumString, sqlx::Type)]
#[sqlx(type_name = "repository_state", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum RepositoryStatus {
    New,
    Active,
    Legacy,
    Readded,
}

#[derive(Clone, Debug, PartialEq, sqlx::Type, Deserialize, Eq, Hash)]
#[sqlx(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Repository,
    Modules,
    Site,
}

#[derive(Debug, Clone, FromRow)]
pub struct RepositoryData {
    pub id: i16,
    pub name: String,
    pub title: String,
    pub singular: String,
    pub eol_date: Option<chrono::NaiveDate>, // TODO: convert to chrono
    pub status: RepositoryStatus,
    pub source_type: SourceType,
}

#[derive(Default)]
struct CachedData {
    repositories: Vec<RepositoryData>,
    repositories_by_name: HashMap<String, RepositoryData>,
}

pub struct RepositoryDataCache {
    pool: PgPool,
    cached_data: Mutex<Arc<CachedData>>,
}

impl RepositoryDataCache {
    pub async fn new(pool: PgPool) -> Result<Self> {
        Ok(Self {
            cached_data: Mutex::new(Arc::new(Self::fetch(&pool).await?)),
            pool,
        })
    }

    async fn fetch(pool: &PgPool) -> Result<CachedData> {
        // XXX: COALESCE for singular and source_type are meant for
        // legacy repositories which don't have meta properly filled
        let repositories: Vec<RepositoryData> = sqlx::query_as(indoc! {r#"
            SELECT
                id,
                name,
                "desc" AS title,
                COALESCE(metadata->>'singular', name || ' package') AS singular,
                (metadata->>'valid_till')::DATE AS eol_date,
                state AS status,
                COALESCE(metadata->>'type', 'repository') AS source_type
            FROM repositories
            ORDER BY sortname
        "#})
        .fetch_all(pool)
        .await?;

        let repositories_by_name = repositories
            .iter()
            .cloned()
            .map(|repository| (repository.name.clone(), repository))
            .collect();

        Ok(CachedData {
            repositories_by_name,
            repositories,
        })
    }

    pub async fn update(&self) -> Result<()> {
        let data = Self::fetch(&self.pool).await?;
        let count = data.repositories.len();
        let data = Arc::new(data);
        *self.cached_data.lock().unwrap() = data;
        if let Ok(timestamp) = UNIX_EPOCH.elapsed() {
            gauge!("repology_webapp_repository_data_cache_last_update_seconds")
                .set(timestamp.as_secs_f64());
        }
        info!("updated repository data cache, {} entries", count);
        Ok(())
    }

    pub fn get(&self, repository_name: &str) -> Option<RepositoryData> {
        let cached_data = self.cached_data.lock().unwrap().clone();
        cached_data
            .repositories_by_name
            .get(repository_name)
            .cloned()
    }

    pub fn get_active(&self, repository_name: &str) -> Option<RepositoryData> {
        let cached_data = self.cached_data.lock().unwrap().clone();
        cached_data
            .repositories_by_name
            .get(repository_name)
            .filter(|metadata| metadata.status == RepositoryStatus::Active)
            .cloned()
    }

    pub fn get_all_active(&self) -> Vec<RepositoryData> {
        let cached_data = self.cached_data.lock().unwrap().clone();
        cached_data
            .repositories
            .iter()
            .filter(|metadata| metadata.status == RepositoryStatus::Active)
            .cloned()
            .collect()
    }
}
