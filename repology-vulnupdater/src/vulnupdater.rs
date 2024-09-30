// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use anyhow::Error;
use chrono::{DateTime, TimeDelta, Utc};
use tracing::{info, instrument, warn};

use crate::fetcher::{NvdFetcher, Paginator};
use crate::processors::DatasourceProcessor;
use crate::status_tracker::SourceUpdateStatusTracker;

// according to https://nvd.nist.gov/developers/vulnerabilities:
// "The maximum allowable range when using any date range parameters is 120 consecutive days"
const MAX_INCREMENTAL_UPDATE_SPAN: TimeDelta = TimeDelta::days(120);

// when doing delta updates, add some overlap to periods, because NVD API
// relies on times and is not reliable
const INCREMENTAL_UPDATE_OVERLAP: Duration = Duration::from_hours(2);

pub struct Datasource<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub processor: &'a dyn DatasourceProcessor,
}

pub struct VulnUpdater<'a> {
    status_tracker: &'a SourceUpdateStatusTracker<'a>,
    fetcher: &'a NvdFetcher,
}

impl<'a> VulnUpdater<'a> {
    pub fn new(status_tracker: &'a SourceUpdateStatusTracker, fetcher: &'a NvdFetcher) -> Self {
        Self {
            status_tracker,
            fetcher,
        }
    }

    async fn setup_fresh_full_update(
        &self,
        datasource: &Datasource<'a>,
        now: DateTime<Utc>,
    ) -> Result<Paginator, Error> {
        let pager = self.fetcher.paginate(datasource.url);
        self.status_tracker
            .register_update_attempt(datasource.name, now)
            .await?;
        Ok(pager)
    }

    async fn setup_continued_full_update(
        &self,
        datasource: &Datasource<'a>,
        offset: u64,
    ) -> Result<Paginator, Error> {
        let mut pager = self.fetcher.paginate(datasource.url);
        pager.seek(offset);
        Ok(pager)
    }

    async fn setup_incremental_update(
        &self,
        datasource: &Datasource<'a>,
        now: DateTime<Utc>,
        since: DateTime<Utc>,
    ) -> Result<Paginator, Error> {
        let start_date = (since - INCREMENTAL_UPDATE_OVERLAP).min(now - INCREMENTAL_UPDATE_OVERLAP);
        let end_date = now + INCREMENTAL_UPDATE_OVERLAP;
        if (end_date - start_date) > MAX_INCREMENTAL_UPDATE_SPAN {
            warn!(
                "incremental period too big ({days} days), falling back to full update",
                days = (end_date - start_date).num_days()
            );
            return self.setup_fresh_full_update(datasource, now).await;
        }
        const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
        let url = format!(
            "{}?lastModStartDate={}&lastModEndDate={}",
            datasource.url,
            start_date.format(DATETIME_FORMAT),
            end_date.format(DATETIME_FORMAT)
        );
        let pager = self.fetcher.paginate(&url);
        self.status_tracker
            .register_update_attempt(datasource.name, now)
            .await?;
        Ok(pager)
    }

    #[instrument("datasource", skip_all, fields(datasource=datasource.name), err)]
    async fn update_datasource(
        &self,
        datasource: &Datasource<'a>,
        update_period: Option<Duration>,
    ) -> Result<Option<DateTime<Utc>>, Error> {
        let source_status = self.status_tracker.get(datasource.name).await?;

        let now = Utc::now();
        let (is_full_update, mut pager) =
            if let Some(offset) = source_status.current_full_update_offset {
                info!("continuing full update at {offset}");
                (
                    true,
                    self.setup_continued_full_update(datasource, offset as u64)
                        .await?,
                )
            } else if let Some(last_update_time) = source_status.last_update_time {
                if let Some(update_period) = update_period {
                    let deadline = last_update_time + update_period;
                    if now < deadline {
                        info!("update time has not come yet, sheduled at {deadline:?}");
                        return Ok(Some(deadline));
                    }
                }
                info!("incremental update since {last_update_time}");
                (
                    false,
                    self.setup_incremental_update(datasource, now, last_update_time)
                        .await?,
                )
            } else {
                info!("starting full update");
                (true, self.setup_fresh_full_update(datasource, now).await?)
            };

        let mut num_changes = 0;
        while let Some(page) = pager.fetch_next().await? {
            let process_status = datasource.processor.process(&page).await?;
            num_changes += process_status.num_changes;
            if is_full_update {
                self.status_tracker
                    .register_full_update_progress(datasource.name, pager.current_offset())
                    .await?;
            }
            info!(
                "processed {current_offset} of {total_results}",
                current_offset = pager.current_offset(),
                total_results = pager.total_results().unwrap_or(0)
            );
        }

        info!("finalizing");
        datasource.processor.finalize().await?;

        self.status_tracker
            .register_successful_update(datasource.name, is_full_update)
            .await?;

        info!(num_changes = num_changes);

        Ok(update_period.map(|period| now + period))
    }

    #[instrument("oneshot", skip_all)]
    pub async fn process_datasources_once(
        &self,
        datasources: &[Datasource<'a>],
    ) -> Result<(), Error> {
        for datasource in datasources {
            self.update_datasource(datasource, None).await?;
        }
        Ok(())
    }

    #[instrument("loop", skip_all, fields(period = ?update_period))]
    pub async fn run_loop(&self, datasources: &[Datasource<'a>], update_period: Duration) {
        loop {
            info!("start loop iteration");
            let mut next_update = Utc::now() + update_period;

            for datasource in datasources {
                if let Ok(Some(source_next_update)) = self
                    .update_datasource(datasource, Some(update_period))
                    .await
                {
                    next_update = next_update.min(source_next_update);
                }
            }

            if let Ok(sleep_duration) = (next_update - Utc::now()).to_std() {
                info!("sleeping for {sleep_duration:?} before next iteration");
                tokio::time::sleep(sleep_duration).await;
            }
        }
    }
}
