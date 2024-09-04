// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Error;
use serde::Deserialize;
use sqlx::{FromRow, PgPool};
use strum_macros::EnumString;

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
pub struct RepositoryMetadata {
    pub name: String,
    pub title: String,
    pub singular: String,
    #[allow(dead_code)]
    pub eol_date: Option<chrono::NaiveDate>, // TODO: convert to chrono
    pub status: RepositoryStatus,
    pub source_type: SourceType,
}

#[derive(Default)]
struct CachedData {
    last_update: Option<Instant>,
    // XXX: Wrap RepositoryMetadata into Arc<>, which would allow to
    // avoid data duplication both when storing metadata here, and when
    // returning it from get_* methods
    repositories: Vec<RepositoryMetadata>,
    repositories_by_name: HashMap<String, RepositoryMetadata>,
}

#[derive(Clone)]
pub struct RepositoryMetadataCache {
    pool: PgPool,
    cached_data: Arc<Mutex<CachedData>>,
}

const CACHE_DURATION: Duration = Duration::from_secs(300);

impl RepositoryMetadataCache {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cached_data: Default::default(),
        }
    }

    pub async fn update(&self) -> Result<(), Error> {
        sqlx::query_as(
            r#"
            SELECT
                name,
                "desc" AS title,
                metadata->>'singular' AS singular,
                (metadata->>'valid_till')::DATE AS eol_date,
                state AS status,
                metadata->>'type' AS source_type
            FROM repositories
            ORDER BY sortname
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .and_then(|repositories: Vec<RepositoryMetadata>| {
            let cached_data = &mut self.cached_data.lock().unwrap();
            cached_data.repositories_by_name = repositories
                .iter()
                .cloned()
                .map(|repository| (repository.name.clone(), repository))
                .collect();
            cached_data.repositories = repositories;
            cached_data.last_update = Some(Instant::now());
            Ok(())
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

    pub async fn get(&self, repository_name: &str) -> Option<RepositoryMetadata> {
        self.try_update_if_needed().await;
        self.cached_data
            .lock()
            .unwrap()
            .repositories_by_name
            .get(repository_name)
            .cloned()
    }

    pub async fn get_active(&self, repository_name: &str) -> Option<RepositoryMetadata> {
        self.try_update_if_needed().await;
        self.cached_data
            .lock()
            .unwrap()
            .repositories_by_name
            .get(repository_name)
            .filter(|metadata| metadata.status == RepositoryStatus::Active)
            .cloned()
    }

    pub async fn get_all_active(&self) -> Vec<RepositoryMetadata> {
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
