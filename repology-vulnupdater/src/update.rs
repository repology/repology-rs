// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use anyhow::{bail, Error};
use async_compression::tokio::bufread::GzipDecoder;
use reqwest::StatusCode;
use sqlx::{FromRow, PgPool};
use tokio::io::BufReader;
use tokio_stream::StreamExt;

use crate::datasources::Datasource;
use crate::processors::DatasourceUpdateResult;

const USER_AGENT: &str = "repology-vulnupdater/1 (+https://repology.org/docs/bots)";
const TIMEOUT: Duration = Duration::from_mins(60);

#[derive(FromRow, Default)]
pub struct DatasourceState {
    pub etag: Option<String>,
    pub age: Option<sqlx::postgres::types::PgInterval>, // TODO: change do Duration, handle deserialization
}

pub struct Updater {
    pool: PgPool,
    client: reqwest::Client,
}

impl Updater {
    pub fn new(pool: PgPool) -> Result<Self, Error> {
        Ok(Self {
            pool,
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .connect_timeout(TIMEOUT)
                .read_timeout(TIMEOUT)
                .build()?,
        })
    }

    async fn load_source_state(&self, datasource: &Datasource) -> DatasourceState {
        sqlx::query_as(
            r#"
            SELECT
                etag,
                now() - last_update AS age
            FROM vulnerability_sources
            WHERE url = $1
            "#,
        )
        .bind(&datasource.url)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or_default()
        .unwrap_or_default()
    }

    async fn update_source_state(
        &self,
        datasource: &Datasource,
        new_etag: Option<&str>,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO vulnerability_sources(
                url,
                etag,
                last_update,
                total_updates,
                type
            )
            VALUES (
                $1,
                $2,
                now(),
                1,
                ''
            )
            ON CONFLICT(url) DO UPDATE
            SET
                etag = coalesce($2, vulnerability_sources.etag),
                last_update = now(),
                total_updates = vulnerability_sources.total_updates + 1
            "#,
        )
        .bind(&datasource.url)
        .bind(new_etag)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_source(
        &self,
        datasource: &Datasource,
    ) -> Result<DatasourceUpdateResult, Error> {
        eprintln!("source {}: start update", datasource.url);

        let res: Result<DatasourceUpdateResult, Error> = try {
            let state = self.load_source_state(datasource).await;

            if let Some(age) = state.age {
                let age = Duration::from_micros(age.microseconds.try_into().unwrap());
                if age < datasource.update_period {
                    return Ok(DatasourceUpdateResult::NoUpdateNeededFor(
                        datasource.update_period - age,
                    ));
                }
            }

            let mut request = self.client.get(&datasource.url);
            if let Some(etag) = &state.etag {
                request = request.header("if-none-match", etag);
            }
            let response = request.send().await?;

            match response.status() {
                StatusCode::OK => {
                    let new_etag = response
                        .headers()
                        .get("etag")
                        .and_then(|etag| etag.to_str().ok().map(|etag| etag.to_string()));
                    let stream = response.bytes_stream().map(|result| {
                        result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                    });
                    let reader = BufReader::new(GzipDecoder::new(BufReader::new(
                        tokio_util::io::StreamReader::new(stream),
                    )));

                    let res = datasource.processor.process(Box::new(reader)).await?;

                    self.update_source_state(datasource, new_etag.as_deref())
                        .await?;
                    eprintln!(
                        "source {}: update finished ({} updates)",
                        datasource.url,
                        if let DatasourceUpdateResult::HadChanges(num) = res {
                            num
                        } else {
                            0
                        }
                    );
                    res
                }
                StatusCode::NOT_MODIFIED => {
                    self.update_source_state(datasource, state.etag.as_deref())
                        .await?;
                    eprintln!("source {}: not modified", datasource.url);
                    DatasourceUpdateResult::NoChanges
                }
                other => bail!("got bad HTTP code {}", other),
            }
        };

        if let Err(err) = &res {
            eprintln!("source {}: update failed, {}", datasource.url, err);
        }
        res
    }
}
