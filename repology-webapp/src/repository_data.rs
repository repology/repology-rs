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

/// Information on a single repository.
#[derive(Debug, Clone, FromRow)]
pub struct RepositoryData {
    pub id: i16,
    pub name: String,
    pub title: String,
    pub singular: String,
    pub eol_date: Option<chrono::NaiveDate>,
    pub status: RepositoryStatus,
    pub source_type: SourceType,
    pub order: i16,
    pub brand_color: Option<String>,
    pub num_projects_newest: i32,
}

/// Ready to use collection of Repository metadata.
///
/// This is retrieved from [RepositoriesDataCache::snapshot()] and the
/// can be used to query specific bits of Repository data.
pub struct RepositoriesDataSnapshot {
    repositories: Vec<RepositoryData>,
    repositories_by_name: HashMap<String, RepositoryData>,
}

impl RepositoriesDataSnapshot {
    /// Returns repository title by its name.
    ///
    /// Mostly for use in templates - returns human-readable Repository
    /// _title_ by its internal _name_. Never fails - if repository name
    /// is not known, returns it back. This may happen if new repository
    /// was just added but [`RepositoriesDataCache`] has not yet updated
    /// and picked it up.
    pub fn repository_title<'a>(&'a self, repository_name: &'a str) -> &'a str {
        self.repositories_by_name
            .get(repository_name)
            .map(|data| data.title.as_str())
            .unwrap_or(repository_name)
    }

    /// Returns repository data by its name.
    ///
    /// Note that this returns data in non-active (`legacy`) repositories
    /// as well, and in most cases you need [`active_repository`] instead.
    ///
    /// [`active_repository`]: Self::active_repository
    pub fn repository(&self, repository_name: &str) -> Option<&RepositoryData> {
        self.repositories_by_name.get(repository_name)
    }

    /// Checks whether given repository is known and active by its name.
    pub fn is_repository_active(&self, repository_name: &str) -> bool {
        self.active_repository(repository_name).is_some()
    }

    /// Returns repository data for active repository by its name.
    ///
    /// Same as [`repository`], but pretends that Repository does not
    /// exist if it's not in `active` state (in most cases that would be
    /// Repositories which existed at some point, but were then removed
    /// from Repology).
    ///
    /// [`repository`]: Self::repository
    pub fn active_repository(&self, repository_name: &str) -> Option<&RepositoryData> {
        self.repository(repository_name)
            .filter(|data| data.status == RepositoryStatus::Active)
    }

    /// Returns data on all active Repositories.
    ///
    /// Repositories are sorted according to their `sortname`, so this may
    /// be iterated and presented to user right away.
    pub fn active_repositories(&self) -> impl Iterator<Item = &RepositoryData> {
        self.repositories
            .iter()
            .filter(|data| data.status == RepositoryStatus::Active)
    }

    /// Sorts a slice of objects tied to Repositories according to designated Repository order.
    ///
    /// This is useful if a subset if a set of objects related somehow
    /// to Repositories (for instance, per-repository counters) needs to be sorted
    /// according to designated Repository order (based on `sortnames`) to be presented
    /// to user.
    ///
    /// This handles unknown repository names by placing them after all known
    /// repositories, in alphabetical order.
    ///
    /// Note that this implementation avoids `N*logN` HashMap lookups which would
    /// naive approach do:
    ///
    /// ```text
    /// items.sort_by(
    ///     |a, b| {
    ///         self.repositories_by_name.get(get_name(a))...
    ///         self.repositories_by_name.get(get_name(b))...
    ///     }
    /// );
    /// ```
    ///
    /// Instead, it extracts sorting keys in form of (int, &str) pairs,
    /// sorts these, and then places original objects according to that
    /// order. It was not tested if it's actually faster or the performance
    /// is relevant here at all, just that felt right :)
    pub fn sort_by_repository_names<T, F>(&self, items: &mut [T], get_name: F)
    where
        F: Fn(&T) -> &str,
    {
        let keys: Vec<_> = items
            .iter()
            .map(|item| {
                (
                    self.repositories_by_name
                        .get(get_name(item))
                        .map(|data| data.order)
                        .unwrap_or(i16::MAX),
                    get_name(item),
                )
            })
            .collect();
        permutation::permutation::sort(keys).apply_slice_in_place(items);
    }

    /// Sorts a slice of Repository names according to their designated order.
    ///
    /// This is useful if a subset of Repository names must be presented to user,
    /// and we want it to be ordered according to Repositories `sortname`s.
    ///
    /// This handles unknown repository names by placing them after all known
    /// repositories, in alphabetical order.
    pub fn sort_repository_names<T>(&self, names: &mut [T])
    where
        T: AsRef<str>,
    {
        self.sort_by_repository_names(names, |name| name.as_ref());
    }
}

/// Shared facility which stores and periodically updates Repository metadata.
pub struct RepositoriesDataCache {
    pool: PgPool,
    cached_data: Mutex<Arc<RepositoriesDataSnapshot>>,
}

impl RepositoriesDataCache {
    pub async fn new(pool: PgPool) -> Result<Self> {
        Ok(Self {
            cached_data: Mutex::new(Arc::new(Self::fetch(&pool).await?)),
            pool,
        })
    }

    async fn fetch(pool: &PgPool) -> Result<RepositoriesDataSnapshot> {
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
                COALESCE(metadata->>'type', 'repository') AS source_type,
                (row_number() OVER (ORDER BY sortname))::SMALLINT AS order,
                metadata->>'color' AS brand_color,
                num_metapackages_newest AS num_projects_newest
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

        Ok(RepositoriesDataSnapshot {
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

    pub fn snapshot(&self) -> Arc<RepositoriesDataSnapshot> {
        self.cached_data.lock().unwrap().clone()
    }
}
