// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use indoc::indoc;
use maud::{Markup, html};
use sqlx::PgPool;

use crate::components::cpe_item::CpeItem;
use crate::config::Config;

const MAX_CPES: usize = 100;

pub struct CpeItems {
    cpes: Vec<CpeItem>,
}

impl CpeItems {
    pub async fn fetch(pool: &PgPool, key: &str) -> Result<Self> {
        let cpes: Vec<CpeItem> = sqlx::query_as(indoc! {"
            WITH cpes AS (
                SELECT
                    id,
                    effname,
                    cpe_vendor,
                    cpe_product,
                    cpe_edition,
                    cpe_lang,
                    cpe_sw_edition,
                    cpe_target_sw,
                    cpe_target_hw,
                    cpe_other
                FROM manual_cpes
                WHERE
                    effname LIKE ('%' || $1 || '%')
                    OR cpe_vendor LIKE ('%' || $1 || '%')
                    OR cpe_product LIKE ('%' || $1 || '%')
                ORDER BY effname, cpe_vendor, cpe_product, cpe_edition, cpe_lang, cpe_sw_edition, cpe_target_sw, cpe_target_hw, cpe_other
                LIMIT $2
            )
            SELECT
                *,
                EXISTS (
                    SELECT * FROM metapackages WHERE effname = cpes.effname AND num_repos > 0
                ) AS has_alive_project,
                EXISTS (
                    SELECT * FROM project_redirects AS old INNER JOIN project_redirects AS new USING(repository_id, trackname)
                    WHERE NOT old.is_actual AND new.is_actual AND old.project_id = (SELECT id FROM metapackages WHERE effname = cpes.effname)
                ) AS has_project_redirect,
                EXISTS (
                    SELECT * FROM vulnerable_cpes WHERE
                        cpe_vendor = cpes.cpe_vendor AND
                        cpe_product = cpes.cpe_product AND
                        coalesce(nullif(cpe_edition, '*') = nullif(cpes.cpe_edition, '*'), TRUE) AND
                        coalesce(nullif(cpe_lang, '*') = nullif(cpes.cpe_lang, '*'), TRUE) AND
                        coalesce(nullif(cpe_sw_edition, '*') = nullif(cpes.cpe_sw_edition, '*'), TRUE) AND
                        coalesce(nullif(cpe_target_sw, '*') = nullif(cpes.cpe_target_sw, '*'), TRUE) AND
                        coalesce(nullif(cpe_target_hw, '*') = nullif(cpes.cpe_target_hw, '*'), TRUE) AND
                        coalesce(nullif(cpe_other, '*') = nullif(cpes.cpe_other, '*'), TRUE)
                ) AS has_cves,
                EXISTS (
                    SELECT * FROM cpe_dictionary WHERE
                        cpe_vendor = cpes.cpe_vendor AND
                        cpe_product = cpes.cpe_product AND
                        coalesce(nullif(cpe_edition, '*') = nullif(cpes.cpe_edition, '*'), TRUE) AND
                        coalesce(nullif(cpe_lang, '*') = nullif(cpes.cpe_lang, '*'), TRUE) AND
                        coalesce(nullif(cpe_sw_edition, '*') = nullif(cpes.cpe_sw_edition, '*'), TRUE) AND
                        coalesce(nullif(cpe_target_sw, '*') = nullif(cpes.cpe_target_sw, '*'), TRUE) AND
                        coalesce(nullif(cpe_target_hw, '*') = nullif(cpes.cpe_target_hw, '*'), TRUE) AND
                        coalesce(nullif(cpe_other, '*') = nullif(cpes.cpe_other, '*'), TRUE)
                ) AS has_dict
            FROM cpes
            ORDER BY effname, cpe_vendor, cpe_product, cpe_edition, cpe_lang, cpe_sw_edition, cpe_target_sw, cpe_target_hw, cpe_other
        "})
        .bind(key)
        .bind((MAX_CPES + 1) as i32)
        .fetch_all(pool)
        .await?;

        Ok(Self { cpes })
    }

    pub fn render(&self, config: &Config) -> Markup {
        html! {
            @for cpe in self.cpes.iter().take(MAX_CPES) {
                (cpe.render(config))
            }
            @if self.cpes.is_empty() {
                tr {
                    td colspan="6" .has-text-centered {
                        "No results"
                    }
                }
            } @else if self.cpes.len() > MAX_CPES {
                tr {
                    td colspan="6" .has-text-centered {
                        "Too many results, only showing first " (MAX_CPES)
                    }
                }
            }
        }
    }
}
