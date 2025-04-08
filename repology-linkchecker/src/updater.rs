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

use crate::config::DEFAULT_DATABASE_RETRY_PERIOD;
use crate::status::HttpStatusWithRedirect;

#[derive(Debug, Default)]
pub struct CheckResult {
    pub id: i32,
    pub check_time: DateTime<Utc>,
    pub next_check: DateTime<Utc>,
    pub ipv4: Option<HttpStatusWithRedirect>,
    pub ipv6: Option<HttpStatusWithRedirect>,
}

pub struct Updater {
    pool: PgPool,
    dry_run: bool,
    database_retry_period: Duration,
}

impl Updater {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            dry_run: false,
            database_retry_period: DEFAULT_DATABASE_RETRY_PERIOD,
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

    pub async fn try_push(&self, result: &CheckResult) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }

        sqlx::query(indoc! {"
            UPDATE links
            SET
                last_checked = $2,
                next_check = $3,

                ipv4_last_success = CASE WHEN     $4 THEN $2 ELSE ipv4_last_success END,
                ipv4_last_failure = CASE WHEN NOT $4 THEN $2 ELSE ipv4_last_failure END,
                ipv4_success = $4,
                ipv4_status_code = $5,
                ipv4_permanent_redirect_target = $6,

                ipv6_last_success = CASE WHEN     $7 THEN $2 ELSE ipv6_last_success END,
                ipv6_last_failure = CASE WHEN NOT $7 THEN $2 ELSE ipv6_last_failure END,
                ipv6_success = $7,
                ipv6_status_code = $8,
                ipv6_permanent_redirect_target = $9
            WHERE id = $1
        "})
        .bind(result.id)
        .bind(result.check_time)
        .bind(result.next_check)
        .bind(result.ipv4.as_ref().map(|status| status.is_success()))
        .bind(result.ipv4.as_ref().map(|status| status.code()))
        .bind(result.ipv4.as_ref().and_then(|status| status.redirect()))
        .bind(result.ipv6.as_ref().map(|status| status.is_success()))
        .bind(result.ipv6.as_ref().map(|status| status.code()))
        .bind(result.ipv6.as_ref().and_then(|status| status.redirect()))
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
