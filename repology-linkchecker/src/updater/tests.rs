// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![coverage(off)]
#![allow(warnings, unused)]

use std::sync::Mutex;

use anyhow::Result;
use chrono::{DateTime, Utc};
use indoc::indoc;
use serde::Deserialize;
use sqlx::{FromRow, PgPool};

use crate::updater::{CheckResult, Updater};

/*
#[derive(Debug, PartialEq, Eq, FromRow, Default)]
struct Link {
    url: String,
    last_checked: Option<DateTime<Utc>>,
    ipv4_last_success: Option<DateTime<Utc>>,
    ipv4_last_failure: Option<DateTime<Utc>>,
    ipv4_success: Option<bool>,
    ipv4_status_code: Option<i16>,
    ipv4_permanent_redirect_target: Option<String>,
    ipv6_last_success: Option<DateTime<Utc>>,
    ipv6_last_failure: Option<DateTime<Utc>>,
    ipv6_success: Option<bool>,
    ipv6_status_code: Option<i16>,
    ipv6_permanent_redirect_target: Option<String>,
}

impl Link {
    pub async fn fetch(pool: &PgPool, url: &str) -> Result<Self> {
        Ok(sqlx::query_as("SELECT * FROM links WHERE url = $1")
            .bind(url)
            .fetch_one(pool)
            .await?)
    }
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("../../tests/fixtures/example_link.sql")
)]
fn test_result_is_stored(pool: PgPool) {
    let check_time = DateTime::<Utc>::from_timestamp(0, 0).unwrap();

    let check_result = CheckResult {
        url: "https://example.com/".to_string(),
        check_time,
        ipv4: Some(HttpStatus {
            is_success: true,
            status_code: 200,
            permanent_redirect_target: None,
        }),
        ipv6: Some(HttpStatus {
            is_success: false,
            status_code: 404,
            permanent_redirect_target: Some("https://example.com/redirect".to_string()),
        }),
    };

    {
        let updater = Updater::new(pool.clone());
        updater.push(check_result).await.unwrap();
        updater.flush().await.unwrap();
    }

    pretty_assertions::assert_eq!(
        Link::fetch(&pool, "https://example.com/").await.unwrap(),
        Link {
            url: "https://example.com/".to_string(),
            last_checked: Some(check_time),
            ipv4_last_success: Some(check_time),
            ipv4_last_failure: None,
            ipv4_success: Some(true),
            ipv4_status_code: Some(200),
            ipv4_permanent_redirect_target: None,
            ipv6_last_success: None,
            ipv6_last_failure: Some(check_time),
            ipv6_success: Some(false),
            ipv6_status_code: Some(404),
            ipv6_permanent_redirect_target: Some("https://example.com/redirect".to_string()),
            ..Default::default()
        }
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("../../tests/fixtures/example_link.sql")
)]
#[should_panic]
fn test_panic_on_missing_flush(pool: PgPool) {
    let updater = Updater::new(pool);
    updater.push(CheckResult::default()).await.unwrap();
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("../../tests/fixtures/example_link.sql")
)]
fn test_auto_flush(pool: PgPool) {
    let updater = Updater::new(pool).with_batch_size(1);
    updater.push(CheckResult::default()).await.unwrap();
}
*/
