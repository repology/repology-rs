// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod cpe_matches;
mod schema;

use anyhow::Result;
use async_trait::async_trait;
use indoc::indoc;
use metrics::counter;
use sqlx::PgPool;

use crate::datetime::parse_utc_datetime;
use crate::processors::{DatasourceProcessStatus, DatasourceProcessor};

use self::cpe_matches::CpeMatches;

pub struct CveProcessor<'a> {
    pool: &'a PgPool,
    skip_finalization: bool,
}

impl<'a> CveProcessor<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            skip_finalization: false,
        }
    }

    pub fn skip_finalization(mut self, skip: bool) -> Self {
        self.skip_finalization = skip;
        self
    }
}

#[async_trait]
impl DatasourceProcessor for CveProcessor<'_> {
    async fn process(&self, data: &str) -> Result<DatasourceProcessStatus> {
        counter!("repology_vulnupdater_processor_runs_total", "processor" => "cve", "stage" => "processing").increment(1);
        counter!("repology_vulnupdater_processor_data_bytes_total", "processor" => "cve")
            .increment(data.len() as u64);

        let root = serde_json::from_str::<self::schema::Root>(data)?;

        let mut num_changes = 0;

        let mut tx = self.pool.begin().await?;

        for vulnerability in &root.vulnerabilities {
            let cve = &vulnerability.cve;

            let cve_id = cve.id;
            let published = parse_utc_datetime(cve.published)?;
            let last_modified = parse_utc_datetime(cve.last_modified)?;

            let matches = CpeMatches::from_cve(cve);

            let vendor_product_pairs = matches.vendor_product_pairs_for_sql();
            let matches_as_json = matches.into_matches_for_sql();

            let num_rows = sqlx::query(indoc! {"
                WITH updated_cves AS (
                    INSERT INTO cves (
                        cve_id,
                        published,
                        last_modified,
                        matches,
                        cpe_pairs
                    )
                    VALUES (
                        $1,
                        $2,
                        $3,
                        $4,
                        $5
                    )
                    ON CONFLICT(cve_id) DO UPDATE
                    SET
                        published = $2,  -- not expected to change in fact
                        last_modified = $3,
                        matches = $4,
                        cpe_pairs = $5
                    WHERE
                        $3 > cves.last_modified
                    RETURNING cpe_pairs
                )
                INSERT INTO cve_updates (
                    cpe_vendor,
                    cpe_product
                )
                SELECT DISTINCT
                    split_part(unnest(cpe_pairs), ':', 1) AS cpe_vendor,
                    split_part(unnest(cpe_pairs), ':', 2) AS cpe_product
                FROM
                    updated_cves
            "})
            .bind(cve_id)
            .bind(published)
            .bind(last_modified)
            .bind(
                if matches_as_json.as_array().is_some_and(|v| !v.is_empty()) {
                    Some(matches_as_json)
                } else {
                    None
                },
            )
            .bind(if !vendor_product_pairs.is_empty() {
                Some(vendor_product_pairs)
            } else {
                None
            })
            .execute(&mut *tx)
            .await?
            .rows_affected();

            counter!("repology_vulnupdater_processor_sql_rows_total", "processor" => "cve", "operation" => "UPSERT", "stage" => "processing").increment(num_rows);
            num_changes += num_rows;
        }

        tx.commit().await?;

        counter!("repology_vulnupdater_processor_runs_succeeded_total", "processor" => "cve", "stage" => "processing").increment(1);
        Ok(DatasourceProcessStatus { num_changes })
    }

    async fn finalize(&self) -> Result<()> {
        if self.skip_finalization {
            return Ok(());
        }

        counter!("repology_vulnupdater_processor_runs_total", "processor" => "cve", "stage" => "finalization").increment(1);

        let mut tx = self.pool.begin().await?;

        // XXX: switch to MERGE
        let num_rows = sqlx::query(indoc! {"
            DELETE FROM public.vulnerable_cpes
        "})
        .execute(&mut *tx)
        .await?
        .rows_affected();

        counter!("repology_vulnupdater_processor_sql_rows_total", "processor" => "cve", "operation" => "DELETE", "stage" => "finalization", "table" => "vulnerable_cpes").increment(num_rows);

        let num_rows = sqlx::query(indoc! {"
            WITH expanded_matches AS (
                SELECT
                    jsonb_array_elements(matches)->>0 AS cpe_vendor,
                    jsonb_array_elements(matches)->>1 AS cpe_product,
                    jsonb_array_elements(matches)->>2 AS cpe_edition,
                    jsonb_array_elements(matches)->>3 AS cpe_lang,
                    jsonb_array_elements(matches)->>4 AS cpe_sw_edition,
                    jsonb_array_elements(matches)->>5 AS cpe_target_sw,
                    jsonb_array_elements(matches)->>6 AS cpe_target_hw,
                    jsonb_array_elements(matches)->>7 AS cpe_other,
                    jsonb_array_elements(matches)->>8 AS start_version,
                    jsonb_array_elements(matches)->>9 AS end_version,
                    (jsonb_array_elements(matches)->>10)::boolean AS start_version_excluded,
                    (jsonb_array_elements(matches)->>11)::boolean AS end_version_excluded
                FROM cves
            ), matches_with_covering_ranges AS (
                SELECT
                    cpe_vendor,
                    cpe_product,
                    cpe_edition,
                    cpe_lang,
                    cpe_sw_edition,
                    cpe_target_sw,
                    cpe_target_hw,
                    cpe_other,

                    start_version,
                    end_version,
                    start_version_excluded,
                    end_version_excluded,
                    max(end_version::public.versiontext) FILTER(WHERE start_version IS NULL) OVER (
                        PARTITION BY
                            cpe_vendor,
                            cpe_product,
                            cpe_edition,
                            cpe_lang,
                            cpe_sw_edition,
                            cpe_target_sw,
                            cpe_target_hw,
                            cpe_other
                    ) AS covering_end_version
                FROM expanded_matches
            )
            INSERT INTO public.vulnerable_cpes(
                cpe_vendor,
                cpe_product,
                cpe_edition,
                cpe_lang,
                cpe_sw_edition,
                cpe_target_sw,
                cpe_target_hw,
                cpe_other,

                start_version,
                end_version,
                start_version_excluded,
                end_version_excluded
            )
            SELECT DISTINCT
                cpe_vendor,
                cpe_product,
                cpe_edition,
                cpe_lang,
                cpe_sw_edition,
                cpe_target_sw,
                cpe_target_hw,
                cpe_other,

                start_version,
                end_version,
                start_version_excluded,
                end_version_excluded
            FROM matches_with_covering_ranges
            WHERE
                coalesce(public.version_compare2(end_version, covering_end_version) >= 0, true) AND
                end_version IS NOT NULL
        "})
        .execute(&mut *tx)
        .await?
        .rows_affected();

        counter!("repology_vulnupdater_processor_sql_rows_total", "processor" => "cve", "operation" => "INSERT", "stage" => "finalization", "table" => "vulnerable_cpes").increment(num_rows);

        let num_rows = sqlx::query(indoc! {"
            WITH deleted AS (DELETE FROM cve_updates RETURNING cpe_vendor, cpe_product)
            INSERT INTO public.cpe_updates(cpe_vendor, cpe_product) SELECT * FROM deleted;
        "})
        .execute(&mut *tx)
        .await?
        .rows_affected();

        counter!("repology_vulnupdater_processor_sql_rows_total", "processor" => "cve", "operation" => "DELETE", "stage" => "finalization", "table" => "cve_updates").increment(num_rows);
        counter!("repology_vulnupdater_processor_sql_rows_total", "processor" => "cve", "operation" => "INSERT", "stage" => "finalization", "table" => "cpe_updates").increment(num_rows);

        // XXX: switch to MERGE
        let num_rows = sqlx::query(indoc! {"
            DELETE FROM public.cves
        "})
        .execute(&mut *tx)
        .await?
        .rows_affected();

        counter!("repology_vulnupdater_processor_sql_rows_total", "processor" => "cve", "operation" => "DELETE", "stage" => "finalization", "table" => "cves").increment(num_rows);

        let num_rows = sqlx::query(indoc! {"
            INSERT INTO public.cves
            SELECT * FROM cves
        "})
        .execute(&mut *tx)
        .await?
        .rows_affected();

        counter!("repology_vulnupdater_processor_sql_rows_total", "processor" => "cve", "operation" => "INSERT", "stage" => "finalization", "table" => "cves").increment(num_rows);

        tx.commit().await?;

        counter!("repology_vulnupdater_processor_runs_succeeded_total", "processor" => "cve", "stage" => "finalization").increment(1);
        Ok(())
    }
}
