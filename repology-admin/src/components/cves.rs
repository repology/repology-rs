// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use indoc::indoc;
use maud::{Markup, html};
use sqlx::PgPool;

use crate::components::cve::Cve;
use crate::config::Config;

const PERIOD_PER_PAGE: Duration = Duration::from_days(1);

pub struct Cves {
    cves: Vec<Cve>,
    from: DateTime<Utc>,
}

impl Cves {
    pub async fn fetch(pool: &PgPool, to: DateTime<Utc>) -> Result<Self> {
        let from = to - PERIOD_PER_PAGE;

        let cves: Vec<Cve> = sqlx::query_as(indoc! {"
            WITH latest_modified_cves AS (
                SELECT
                    cve_id,
                    last_modified,
                    matches
                FROM cves
                WHERE
                    cpe_pairs IS NOT NULL
                    AND last_modified > $1
                    AND last_modified <= $2
            ), cves_expanded AS (
                SELECT
                    cve_id,
                    last_modified,
                    jsonb_array_elements(matches)->>0 AS cpe_vendor,
                    jsonb_array_elements(matches)->>1 AS cpe_product,
                    jsonb_array_elements(matches)->>2 AS cpe_edition,
                    jsonb_array_elements(matches)->>3 AS cpe_lang,
                    jsonb_array_elements(matches)->>4 AS cpe_sw_edition,
                    jsonb_array_elements(matches)->>5 AS cpe_target_sw,
                    jsonb_array_elements(matches)->>6 AS cpe_target_hw,
                    jsonb_array_elements(matches)->>7 AS cpe_other
                FROM latest_modified_cves
            ), cves_expanded_unmatched AS (
                SELECT
                    *
                FROM cves_expanded
                WHERE
                    NOT EXISTS (
                        SELECT *
                        FROM manual_cpes
                        WHERE
                            cpe_vendor = cves_expanded.cpe_vendor AND
                            cpe_product = cves_expanded.cpe_product AND
                            coalesce(nullif(cpe_edition, '*') = nullif(cves_expanded.cpe_edition, '*'), TRUE) AND
                            coalesce(nullif(cpe_lang, '*') = nullif(cves_expanded.cpe_lang, '*'), TRUE) AND
                            coalesce(nullif(cpe_sw_edition, '*') = nullif(cves_expanded.cpe_sw_edition, '*'), TRUE) AND
                            coalesce(nullif(cpe_target_sw, '*') = nullif(cves_expanded.cpe_target_sw, '*'), TRUE) AND
                            coalesce(nullif(cpe_target_hw, '*') = nullif(cves_expanded.cpe_target_hw, '*'), TRUE) AND
                            coalesce(nullif(cpe_other, '*') = nullif(cves_expanded.cpe_other, '*'), TRUE)
                    )
            )
            SELECT
                max(last_modified) AS last_modified,
                cpe_vendor,
                cpe_product,
                cpe_edition,
                cpe_lang,
                cpe_sw_edition,
                cpe_target_sw,
                cpe_target_hw,
                cpe_other,
                array_agg(
                    DISTINCT cve_id
                    ORDER BY cve_id
                ) AS cve_ids,
                coalesce(
                    (
                        SELECT array_agg(effname) FROM (
                            SELECT effname FROM metapackages WHERE effname_prefixless = replace(lower(cpe_product), '_', '-') AND num_repos > 0
                            UNION
                            SELECT effname FROM metapackages WHERE effname_prefixless = regexp_replace(lower(cpe_product), '[._-]', '')
                            UNION
                            SELECT effname FROM metapackages WHERE effname_prefixless = replace(lower(cpe_vendor || '-' || cpe_product), '_', '-') AND num_repos > 0
                        )
                    ),
                    '{}'::text[]
                ) AS project_candidates
            FROM cves_expanded_unmatched
            GROUP BY
                cpe_vendor,
                cpe_product,
                cpe_edition,
                cpe_lang,
                cpe_sw_edition,
                cpe_target_sw,
                cpe_target_hw,
                cpe_other
            ORDER BY
                last_modified DESC,
                cpe_vendor,
                cpe_product,
                cpe_edition,
                cpe_lang,
                cpe_sw_edition,
                cpe_target_sw,
                cpe_target_hw,
                cpe_other
        "})
        .bind(from)
        .bind(to)
        .fetch_all(pool)
        .await?;

        Ok(Self { cves, from })
    }

    pub fn render(&self, config: &Config) -> Markup {
        html! {
            @for cve in &self.cves {
                (cve.render(config))
            }
            tr {
                td hx-get={"/parts/cves?before=" (self.from.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))} hx-swap="outerHTML" hx-target="closest tr" hx-trigger="revealed" {
                }
            }
        }
    }
}
