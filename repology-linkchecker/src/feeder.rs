// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::{DateTime, Utc};
use indoc::indoc;
use metrics::{counter, gauge};
use sqlx::{FromRow, PgPool};
use tracing::{error, info};

use repology_common::LinkStatus;

use crate::checker::{CheckPriority, CheckTask};
use crate::config::{DEFAULT_BATCH_PERIOD, DEFAULT_BATCH_SIZE, DEFAULT_DATABASE_RETRY_PERIOD};

pub struct Feeder {
    pool: PgPool,
    last_key: Option<(DateTime<Utc>, i32)>,
    last_request_time: Option<Instant>,
    batch_size: usize,
    batch_period: Duration,
    database_retry_period: Duration,
}

#[derive(FromRow)]
struct LinkToCheck {
    id: i32,
    url: String,
    last_checked: Option<DateTime<Utc>>,
    next_check: DateTime<Utc>,
    priority: bool,
    ipv4_status_code: Option<i16>,
    ipv6_status_code: Option<i16>,
    failure_streak: Option<i16>,
}

impl Feeder {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            last_key: None,
            last_request_time: None,
            batch_size: DEFAULT_BATCH_SIZE,
            batch_period: DEFAULT_BATCH_PERIOD,
            database_retry_period: DEFAULT_DATABASE_RETRY_PERIOD,
        }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    pub fn with_batch_period(mut self, batch_period: Duration) -> Self {
        self.batch_period = batch_period;
        self
    }

    pub fn with_database_retry_period(mut self, database_retry_period: Duration) -> Self {
        self.database_retry_period = database_retry_period;
        self
    }

    async fn try_get_next_batch(&mut self) -> Result<Vec<CheckTask>> {
        if let Some(next_request_time) = self
            .last_request_time
            .map(|instant| instant + self.batch_period)
            .filter(|instant| *instant > Instant::now())
        {
            tokio::time::sleep_until(next_request_time.into()).await;
        }
        self.last_request_time = Some(Instant::now());

        let query = if let Some(last_key) = self.last_key {
            sqlx::query_as(indoc! {"
                SELECT
                    id,
                    url,
                    last_checked,
                    next_check,
                    priority,
                    ipv4_status_code,
                    ipv6_status_code,
                    failure_streak
                FROM links
                WHERE
                    refcount > 0
                    AND next_check < now()
                    AND (next_check, id) > ($2, $3)
                ORDER BY next_check, id
                LIMIT $1
            "})
            .bind(self.batch_size as i32)
            .bind(last_key.0)
            .bind(last_key.1)
        } else {
            sqlx::query_as(indoc! {"
                SELECT
                    id,
                    url,
                    last_checked,
                    next_check,
                    priority,
                    ipv4_status_code,
                    ipv6_status_code,
                    failure_streak
                FROM links
                WHERE
                    refcount > 0
                    AND next_check < now()
                ORDER BY next_check, id
                LIMIT $1
            "})
            .bind(self.batch_size as i32)
        };
        let urls: Vec<LinkToCheck> = query.fetch_all(&self.pool).await?;

        counter!("repology_linkchecker_feeder_tasks_total").increment(urls.len() as u64);
        gauge!("repology_linkchecker_feeder_batch_size_total").set(urls.len() as f64);

        self.last_key = if let Some(last_url) = urls.last()
            && urls.len() == self.batch_size
        {
            Some((last_url.next_check, last_url.id))
        } else {
            info!("restarting feeder loop");
            counter!("repology_linkchecker_feeder_loops_total").increment(1);
            None
        };

        info!(
            count = urls.len(),
            oldest = urls.first().map(|link| link.next_check.to_rfc3339()),
            "got batch of check tasks"
        );

        let now = Utc::now();

        if let Some(first_url) = urls.first() {
            gauge!("repology_linkchecker_feeder_oldest_task_age_seconds")
                .set((now - first_url.next_check).as_seconds_f64());
        }

        Ok(urls
            .into_iter()
            .map(|link| CheckTask {
                id: link.id,
                url: link.url,
                priority: if link.priority {
                    CheckPriority::Manual
                } else {
                    CheckPriority::Generated
                },
                last_checked: link.last_checked,
                deadline: link.next_check,
                prev_ipv4_status: link
                    .ipv4_status_code
                    .map(LinkStatus::from_code_with_fallback),
                prev_ipv6_status: link
                    .ipv6_status_code
                    .map(LinkStatus::from_code_with_fallback),
                failure_streak: link
                    .failure_streak
                    .map(|streak| streak.try_into().unwrap_or_default()),
            })
            .collect())
    }

    pub async fn get_next_batch(&mut self) -> Vec<CheckTask> {
        loop {
            match self.try_get_next_batch().await {
                Ok(tasks) => return tasks,
                Err(err) => {
                    error!(
                        ?err,
                        "failed to get updates tasks from the database, will retry"
                    );
                    tokio::time::sleep(self.database_retry_period).await;
                }
            }
        }
    }
}
