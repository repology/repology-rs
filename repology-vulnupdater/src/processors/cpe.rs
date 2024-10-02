// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod schema;

use std::str::FromStr as _;

use anyhow::Error;
use async_trait::async_trait;
use metrics::counter;
use sqlx::PgPool;

use crate::cpe::{Cpe, Part};
use crate::processors::{DatasourceProcessStatus, DatasourceProcessor};

fn cpe_to_json(cpe: Cpe) -> serde_json::Value {
    [
        serde_json::to_value(cpe.vendor).unwrap(),
        serde_json::to_value(cpe.product).unwrap(),
        serde_json::to_value(cpe.version).unwrap(),
        serde_json::to_value(cpe.update).unwrap(),
        serde_json::to_value(cpe.edition).unwrap(),
        serde_json::to_value(cpe.lang).unwrap(),
        serde_json::to_value(cpe.sw_edition).unwrap(),
        serde_json::to_value(cpe.target_sw).unwrap(),
        serde_json::to_value(cpe.target_hw).unwrap(),
        serde_json::to_value(cpe.other).unwrap(),
    ]
    .as_slice()
    .into()
}

pub struct CpeProcessor<'a> {
    pool: &'a PgPool,
    skip_finalization: bool,
}

impl<'a> CpeProcessor<'a> {
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
impl<'a> DatasourceProcessor for CpeProcessor<'a> {
    async fn process(&self, data: &str) -> Result<DatasourceProcessStatus, Error> {
        counter!("repology_vulnupdater_processor_runs_total", "processor" => "cpe", "stage" => "processing").increment(1);
        counter!("repology_vulnupdater_processor_data_bytes_total", "processor" => "cpe")
            .increment(data.len() as u64);

        let root = serde_json::from_str::<self::schema::Root>(data)?;

        let mut add_batch: Vec<Cpe> = vec![];
        let mut delete_batch: Vec<Cpe> = vec![];

        for product in &root.products {
            let cpe = match Cpe::from_str(&product.cpe.cpe_name) {
                Ok(cpe) => cpe,
                Err(_) => {
                    // XXX: log these cases
                    counter!("repology_vulnupdater_processor_products_total", "status" => "skipped", "skip_reason" => "unparsable CPE").increment(1);
                    continue;
                }
            };

            if cpe.part != Part::Applications {
                counter!("repology_vulnupdater_processor_products_total", "status" => "skipped", "skip_reason" => "not applications").increment(1);
                continue;
            }

            if product.cpe.deprecated {
                counter!("repology_vulnupdater_processor_products_total", "status" => "deprecated")
                    .increment(1);
                delete_batch.push(cpe);
            } else {
                counter!("repology_vulnupdater_processor_products_total", "status" => "active")
                    .increment(1);
                add_batch.push(cpe);
            }
        }

        let mut tx = self.pool.begin().await?;

        let num_rows_inserted = sqlx::query(
            r#"
                INSERT INTO cpes (
                    cpe_vendor,
                    cpe_product,
                    cpe_version,
                    cpe_update,
                    cpe_edition,
                    cpe_lang,
                    cpe_sw_edition,
                    cpe_target_sw,
                    cpe_target_hw,
                    cpe_other
                )
                SELECT
                    jsonb_array_elements($1)->>0,
                    jsonb_array_elements($1)->>1,
                    jsonb_array_elements($1)->>2,
                    jsonb_array_elements($1)->>3,
                    jsonb_array_elements($1)->>4,
                    jsonb_array_elements($1)->>5,
                    jsonb_array_elements($1)->>6,
                    jsonb_array_elements($1)->>7,
                    jsonb_array_elements($1)->>8,
                    jsonb_array_elements($1)->>9
                ON CONFLICT DO NOTHING
                "#,
        )
        .bind(
            add_batch
                .into_iter()
                .map(cpe_to_json)
                .collect::<serde_json::Value>(),
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        counter!("repology_vulnupdater_processor_sql_rows_total_total", "processor" => "cpe", "operation" => "INSERT", "stage" => "processing").increment(num_rows_inserted);

        let num_rows_deleted = sqlx::query(
            r#"
                WITH delete_batch AS (
                    SELECT
                        jsonb_array_elements($1)->>0 AS cpe_vendor,
                        jsonb_array_elements($1)->>1 AS cpe_product,
                        jsonb_array_elements($1)->>2 AS cpe_version,
                        jsonb_array_elements($1)->>3 AS cpe_update,
                        jsonb_array_elements($1)->>4 AS cpe_edition,
                        jsonb_array_elements($1)->>5 AS cpe_lang,
                        jsonb_array_elements($1)->>6 AS cpe_sw_edition,
                        jsonb_array_elements($1)->>7 AS cpe_target_sw,
                        jsonb_array_elements($1)->>8 AS cpe_target_hw,
                        jsonb_array_elements($1)->>9 AS cpe_other
                )
                DELETE FROM cpes AS t USING delete_batch AS d WHERE
                    t.cpe_vendor = d.cpe_vendor AND
                    t.cpe_product = d.cpe_product AND
                    t.cpe_version = d.cpe_version AND
                    t.cpe_update = d.cpe_update AND
                    t.cpe_edition = d.cpe_edition AND
                    t.cpe_lang = d.cpe_lang AND
                    t.cpe_sw_edition = d.cpe_sw_edition AND
                    t.cpe_target_sw = d.cpe_target_sw AND
                    t.cpe_target_hw = d.cpe_target_hw AND
                    t.cpe_other = d.cpe_other
                "#,
        )
        .bind(
            delete_batch
                .into_iter()
                .map(cpe_to_json)
                .collect::<serde_json::Value>(),
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        counter!("repology_vulnupdater_processor_sql_rows_total_total", "processor" => "cpe", "operation" => "DELETE", "stage" => "processing").increment(num_rows_deleted);

        tx.commit().await?;

        counter!("repology_vulnupdater_processor_runs_succeeded_total", "processor" => "cpe", "stage" => "processing").increment(1);
        Ok(DatasourceProcessStatus {
            num_changes: num_rows_inserted + num_rows_deleted,
        })
    }

    async fn finalize(&self) -> Result<(), Error> {
        if self.skip_finalization {
            return Ok(());
        }

        counter!("repology_vulnupdater_processor_runs_total", "processor" => "cpe", "stage" => "finalization").increment(1);

        let mut tx = self.pool.begin().await?;

        // XXX: switch to MERGE
        let num_rows = sqlx::query(
            r#"
            DELETE FROM public.cpe_dictionary_test;
            "#,
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        counter!("repology_vulnupdater_processor_sql_rows_total", "processor" => "cpe", "operation" => "DELETE", "stage" => "finalization").increment(num_rows);

        sqlx::query(
            r#"
            INSERT INTO public.cpe_dictionary_test (
                cpe_vendor,
                cpe_product,
                cpe_edition,
                cpe_lang,
                cpe_sw_edition,
                cpe_target_sw,
                cpe_target_hw,
                cpe_other
            )
            SELECT DISTINCT
                cpe_vendor,
                cpe_product,
                cpe_edition,
                cpe_lang,
                cpe_sw_edition,
                cpe_target_sw,
                cpe_target_hw,
                cpe_other
            FROM
                cpes
            "#,
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        counter!("repology_vulnupdater_processor_sql_rows_total_total", "processor" => "cpe", "operation" => "INSERT", "stage" => "finalization").increment(num_rows);

        tx.commit().await?;

        counter!("repology_vulnupdater_processor_runs_succeeded_total", "processor" => "cpe", "stage" => "finalization").increment(1);
        Ok(())
    }
}
