// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
mod tests;

use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use indoc::indoc;
use sqlx::PgPool;
use tracing::error;

use repology_common::LinkStatusWithRedirect;

use crate::config::DEFAULT_DATABASE_RETRY_PERIOD;
use crate::optional_semaphore::OptionalSemaphore;

#[derive(Debug, Default)]
pub struct CheckResult {
    pub id: i32,
    pub check_time: DateTime<Utc>,
    pub next_check: DateTime<Utc>,
    pub ipv4: Option<LinkStatusWithRedirect>,
    pub ipv6: Option<LinkStatusWithRedirect>,
}

impl CheckResult {
    fn is_success(&self) -> Option<bool> {
        match (
            self.ipv4.as_ref().map(|status| status.status.is_success()),
            self.ipv6.as_ref().map(|status| status.status.is_success()),
        ) {
            (None, None) => None,
            (Some(true), _) => Some(true),
            (_, Some(true)) => Some(true),
            _ => Some(false),
        }
    }
}

pub struct Updater {
    pool: PgPool,
    dry_run: bool,
    database_retry_period: Duration,
    update_semaphore: OptionalSemaphore,
}

impl Updater {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            dry_run: false,
            database_retry_period: DEFAULT_DATABASE_RETRY_PERIOD,
            update_semaphore: Default::default(),
        }
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn with_database_retry_period(mut self, database_retry_period: Duration) -> Self {
        self.database_retry_period = database_retry_period;
        self
    }

    pub fn with_max_parallel_updates(mut self, max_parallel_updates: usize) -> Self {
        self.update_semaphore = OptionalSemaphore::new(max_parallel_updates);
        self
    }

    pub async fn try_push(&self, result: &CheckResult) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }

        let _permit = self
            .update_semaphore
            .acquire()
            .await
            .expect("expected to be able to acquire update semaphore");

        sqlx::query(indoc! {"
            UPDATE links
            SET
                last_checked = $2,
                next_check = $3,
                last_success = CASE
                    WHEN $4 IS NULL THEN NULL
                    WHEN $4 THEN $2
                    ELSE last_success
                END,
                last_failure = CASE
                    WHEN $4 IS NULL THEN NULL
                    WHEN NOT $4 THEN $2
                    ELSE last_failure
                END,
                failure_streak = CASE
                    WHEN NOT $4 THEN coalesce(failure_streak, 0) + 1
                    ELSE NULL
                END,
                ipv4_status_code = $5,
                ipv4_permanent_redirect_target = $6,
                ipv6_status_code = $7,
                ipv6_permanent_redirect_target = $8
            WHERE id = $1
        "})
        .bind(result.id) // $1
        .bind(result.check_time) // $2
        .bind(result.next_check) // $3
        .bind(result.is_success()) // $4
        .bind(result.ipv4.as_ref().map(|status| status.code())) // $5
        .bind(result.ipv4.as_ref().and_then(|status| status.redirect())) // $6
        .bind(result.ipv6.as_ref().map(|status| status.code())) // $7
        .bind(result.ipv6.as_ref().and_then(|status| status.redirect())) // $8
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn push(&self, result: CheckResult) {
        loop {
            match self.try_push(&result).await {
                Ok(()) => {
                    return;
                }
                Err(err) => {
                    error!(?err, "failed to write update to the database, will retry");
                    tokio::time::sleep(self.database_retry_period).await;
                }
            }
        }
    }

    pub async fn try_defer_by(&self, id: i32, duration: Duration) -> Result<()> {
        if !self.dry_run {
            let _permit = self
                .update_semaphore
                .acquire()
                .await
                .expect("expected to be able to acquire update semaphore");

            sqlx::query("UPDATE links SET next_check = now() + $2 WHERE id = $1")
                .bind(id)
                .bind(Duration::from_secs(duration.as_secs())) // avoid "does not support nanoseconds precision" erro
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    pub async fn defer_by(&self, id: i32, duration: Duration) {
        loop {
            match self.try_defer_by(id, duration).await {
                Ok(()) => {
                    return;
                }
                Err(err) => {
                    error!(?err, "failed to write update to the database, will retry");
                    tokio::time::sleep(self.database_retry_period).await;
                }
            }
        }
    }
}
