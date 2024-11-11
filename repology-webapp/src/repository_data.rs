// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Error;
use indoc::indoc;
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
    last_update: Option<Instant>,
    // XXX: Wrap RepositoryData into Arc<>, which would allow to
    // avoid data duplication both when storing metadata here, and when
    // returning it from get_* methods
    repositories: Vec<RepositoryData>,
    repositories_by_name: HashMap<String, RepositoryData>,
}

#[derive(Clone)]
pub struct RepositoryDataCache {
    pool: PgPool,
    cached_data: Arc<Mutex<CachedData>>,
}

const CACHE_DURATION: Duration = Duration::from_secs(300);

impl RepositoryDataCache {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cached_data: Default::default(),
        }
    }

    pub async fn update(&self) -> Result<(), Error> {
        // XXX: COALESCE for singular and source_type are meant for
        // legacy repositories which don't have meta properly filled
        sqlx::query_as(indoc! {r#"
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
        .fetch_all(&self.pool)
        .await
        .map(|repositories: Vec<RepositoryData>| {
            let cached_data = &mut self.cached_data.lock().unwrap();
            cached_data.repositories_by_name = repositories
                .iter()
                .cloned()
                .map(|repository| (repository.name.clone(), repository))
                .collect();
            cached_data.repositories = repositories;
            cached_data.last_update = Some(Instant::now());
            info!(
                "updated repository data cache, {} entries",
                cached_data.repositories.len()
            );
        })?;
        Ok(())
    }

    async fn try_update_if_needed(&self) {
        // XXX: instead of updating cache from requests, spawn explicit task
        // to update it regularly. May benefit from switchung Mutex to RWLock
        // in that case
        if !self
            .cached_data
            .lock()
            .unwrap()
            .last_update
            .is_some_and(|t| t.elapsed() < CACHE_DURATION)
        {
            let _ = self.update().await;
        }
    }

    pub async fn get(&self, repository_name: &str) -> Option<RepositoryData> {
        self.try_update_if_needed().await;
        self.cached_data
            .lock()
            .unwrap()
            .repositories_by_name
            .get(repository_name)
            .cloned()
    }

    pub async fn get_active(&self, repository_name: &str) -> Option<RepositoryData> {
        self.try_update_if_needed().await;
        self.cached_data
            .lock()
            .unwrap()
            .repositories_by_name
            .get(repository_name)
            .filter(|metadata| metadata.status == RepositoryStatus::Active)
            .cloned()
    }

    pub async fn get_all_active(&self) -> Vec<RepositoryData> {
        self.try_update_if_needed().await;
        self.cached_data
            .lock()
            .unwrap()
            .repositories
            .iter()
            .filter(|metadata| metadata.status == RepositoryStatus::Active)
            .cloned()
            .collect()
    }
}
