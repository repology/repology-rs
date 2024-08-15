// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Error;
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

#[derive(Debug, Clone, FromRow)]
pub struct RepositoryMetadata {
    pub name: String,
    #[allow(dead_code)]
    pub title: String,
    pub singular: String,
    #[allow(dead_code)]
    pub eol_date: Option<String>, // TODO: convert to chrono
    pub status: RepositoryStatus,
}

#[derive(Default)]
struct CachedData {
    last_update: Option<Instant>,
    repositories: Vec<RepositoryMetadata>,
    repositories_by_name: HashMap<String, RepositoryMetadata>,
}

#[derive(Clone)]
pub struct RepositoryMetadataCache {
    pool: PgPool,
    cached_data: Arc<Mutex<CachedData>>,
}

unsafe impl Send for RepositoryMetadataCache {}

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
                metadata->>'valid_till' AS eol_date,
                state AS status
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
            .map(|metadata| metadata.clone())
    }

    pub async fn get_active(&self, repository_name: &str) -> Option<RepositoryMetadata> {
        self.try_update_if_needed().await;
        self.cached_data
            .lock()
            .unwrap()
            .repositories_by_name
            .get(repository_name)
            .filter(|metadata| metadata.status == RepositoryStatus::Active)
            .map(|metadata| metadata.clone())
    }
}
