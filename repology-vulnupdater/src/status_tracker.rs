// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Error;
use chrono::{DateTime, Utc};
use indoc::indoc;
use sqlx::{FromRow, PgPool};

#[derive(FromRow, Default)]
pub struct SourceUpdateStatus {
    pub current_full_update_offset: Option<i64>,
    #[allow(dead_code)] // we'll use it to force full update occasionally
    pub last_full_update_time: Option<DateTime<Utc>>,
    pub last_update_time: Option<DateTime<Utc>>,
}

pub struct SourceUpdateStatusTracker<'a> {
    pool: &'a PgPool,
}

impl<'a> SourceUpdateStatusTracker<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, name: &str) -> Result<SourceUpdateStatus, Error> {
        Ok(sqlx::query_as(indoc! {"
            SELECT current_full_update_offset, last_full_update_time, last_update_time
            FROM update_status
            WHERE name = $1
        "})
        .bind(name)
        .fetch_optional(self.pool)
        .await?
        .unwrap_or_default())
    }

    pub async fn register_update_attempt(
        &self,
        name: &str,
        time: DateTime<Utc>,
    ) -> Result<(), Error> {
        sqlx::query(indoc! {"
            INSERT INTO update_status(name, last_update_attempt_time)
            VALUES ($1, $2)
            ON CONFLICT(name) DO UPDATE SET last_update_attempt_time = $2
        "})
        .bind(name)
        .bind(time)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn register_successful_update(&self, name: &str, is_full: bool) -> Result<(), Error> {
        sqlx::query(indoc! {"
            UPDATE update_status
            SET
                last_update_time = last_update_attempt_time,
                current_full_update_offset = NULL,
                last_full_update_time = CASE WHEN $2 THEN last_update_attempt_time ELSE last_full_update_time END
            WHERE name = $1
        "})
        .bind(name)
        .bind(is_full)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn register_full_update_progress(
        &self,
        name: &str,
        current_offset: u64,
    ) -> Result<(), Error> {
        sqlx::query(indoc! {"
            UPDATE update_status
            SET current_full_update_offset = $2
            WHERE name = $1
        "})
        .bind(name)
        .bind(current_offset as i64)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
