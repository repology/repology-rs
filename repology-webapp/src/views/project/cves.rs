// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use indexmap::IndexMap;
use indoc::indoc;
use libversion::version_compare;
use serde::Deserialize;
use sqlx::FromRow;

use crate::endpoints::Endpoint;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

use super::common::Project;
use super::nonexistent::nonexisting_project;

fn version_compare_nulls_last(a: &Option<&str>, b: &Option<&str>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(a), Some(b)) => version_compare(a, b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn version_compare_nulls_first(a: &Option<&str>, b: &Option<&str>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(a), Some(b)) => version_compare(a, b),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(rename = "version")]
    pub highlighted_version: Option<String>,
}

#[derive(FromRow)]
struct Cve {
    pub cve_id: String,
    pub published: String,
    pub last_modified: String,

    pub cpe_vendor: String,
    pub cpe_product: String,
    pub cpe_edition: String,
    pub cpe_lang: String,
    pub cpe_sw_edition: String,
    pub cpe_target_sw: String,
    pub cpe_target_hw: String,
    pub cpe_other: String,

    pub start_version: Option<String>,
    pub end_version: Option<String>,
    pub start_version_excluded: bool,
    pub end_version_excluded: bool,
}

impl Cve {
    fn version_range_text(&self) -> String {
        match (self.start_version.as_deref(), self.end_version.as_deref()) {
            (Some(start), Some(end)) if start == end => start.to_owned(),
            (None, None) => "(-∞, +∞)".to_owned(),
            (start, end) => format!(
                "{}{}, {}{}",
                if start.is_none() || self.start_version_excluded {
                    '('
                } else {
                    '['
                },
                start.unwrap_or("-∞"),
                end.unwrap_or("+∞"),
                if end.is_none() || self.end_version_excluded {
                    ')'
                } else {
                    ']'
                },
            ),
        }
    }

    fn is_version_in_range(&self, version: &str) -> bool {
        self.start_version.as_deref().is_none_or(|start| {
            let cmp = version_compare(start, version);
            cmp.is_lt() || !self.start_version_excluded && cmp.is_le()
        }) && self.end_version.as_deref().is_none_or(|end| {
            let cmp = version_compare(version, end);
            cmp.is_lt() || !self.end_version_excluded && cmp.is_le()
        })
    }

    fn sort_key(&self) -> (u32, u32) {
        if let Some(numbers) = self.cve_id.strip_prefix("CVE-") {
            if let Some((a, b)) = numbers.split_once('-') {
                let a: u32 = a.parse().unwrap_or(0);
                let b: u32 = b.parse().unwrap_or(0);
                return (a, b);
            }
        }
        (0, 0)
    }
}

#[derive(PartialEq, Eq, Hash)]
struct CveAggregation {
    pub cve_id: String,
    pub published: String,
    pub last_modified: String,

    pub cpe_vendor: String,
    pub cpe_product: String,
    pub cpe_edition: String,
    pub cpe_lang: String,
    pub cpe_sw_edition: String,
    pub cpe_target_sw: String,
    pub cpe_target_hw: String,
    pub cpe_other: String,
}

struct CveVersionRange {
    pub text: String,
    pub highlighted: bool,
}

#[derive(Template)]
#[template(path = "project/cves.html")]
struct TemplateParams<'a> {
    ctx: TemplateContext,
    project_name: String,
    project: Option<Project>,
    num_cves: usize,
    highlighted_version: Option<&'a str>,
    aggregated_cves: IndexMap<CveAggregation, Vec<CveVersionRange>>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn project_cves(
    Path(project_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new_without_params(Endpoint::ProjectCves);

    let project: Option<Project> = sqlx::query_as(indoc! {"
        SELECT
            num_repos,
            has_cves,
            has_related,
            orphaned_at
        FROM metapackages
        WHERE effname = $1
    "})
    .bind(&project_name)
    .fetch_optional(&state.pool)
    .await?;

    let mut cves: Vec<Cve> = sqlx::query_as(indoc! {r#"
        SELECT
            cve_id,
            to_char(published::timestamptz at time zone 'UTC', 'YYYY-MM-DD"T"HH24:MI"Z"') AS published,
            to_char(last_modified::timestamptz at time zone 'UTC', 'YYYY-MM-DD"T"HH24:MI"Z"') AS last_modified,

            expanded_cves.cpe_vendor,
            expanded_cves.cpe_product,
            expanded_cves.cpe_edition,
            expanded_cves.cpe_lang,
            expanded_cves.cpe_sw_edition,
            expanded_cves.cpe_target_sw,
            expanded_cves.cpe_target_hw,
            expanded_cves.cpe_other,

            start_version,
            end_version,
            start_version_excluded,
            end_version_excluded
        FROM (
            SELECT
                *
            FROM (
                -- preselect possibly matching CVEs; this may contain unrelated CVEs
                -- for only vendor/product are checked here; they are filtered in the top-level join
                SELECT
                    cve_id,
                    published,
                    last_modified,
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
                    (jsonb_array_elements(matches)->>11)::boolean AS end_version_excluded,
                    row_number() OVER (ORDER BY split_part(cve_id, '-', 2)::integer DESC, split_part(cve_id, '-', 3)::integer DESC) AS rn
                FROM cves
                WHERE cpe_pairs && (
                    SELECT
                        array_agg(cpe_vendor || ':' || cpe_product)
                    FROM manual_cpes
                    WHERE effname = $1
                )
            ) AS tmp
            WHERE rn <= $2
        ) AS expanded_cves
        INNER JOIN manual_cpes ON
            expanded_cves.cpe_vendor = manual_cpes.cpe_vendor AND
            expanded_cves.cpe_product = manual_cpes.cpe_product AND
            coalesce(nullif(expanded_cves.cpe_edition, '*') = nullif(manual_cpes.cpe_edition, '*'), TRUE) AND
            coalesce(nullif(expanded_cves.cpe_lang, '*') = nullif(manual_cpes.cpe_lang, '*'), TRUE) AND
            coalesce(nullif(expanded_cves.cpe_sw_edition, '*') = nullif(manual_cpes.cpe_sw_edition, '*'), TRUE) AND
            coalesce(nullif(expanded_cves.cpe_target_sw, '*') = nullif(manual_cpes.cpe_target_sw, '*'), TRUE) AND
            coalesce(nullif(expanded_cves.cpe_target_hw, '*') = nullif(manual_cpes.cpe_target_hw, '*'), TRUE) AND
            coalesce(nullif(expanded_cves.cpe_other, '*') = nullif(manual_cpes.cpe_other, '*'), TRUE)
        WHERE effname = $1
        -- Sorted in rust code, so we don't need to pull libversion externsion here
        --ORDER BY
        --    -- assuming CVE-xxxx-yyyyy format
        --    split_part(cve_id, '-', 2)::integer,
        --    split_part(cve_id, '-', 3)::integer,
        --    end_version::versiontext NULLS LAST,
        --    start_version::versiontext NULLS FIRST
    "#})
    .bind(&project_name)
    .bind(&(crate::constants::CVES_PER_PAGE as i32))
    .fetch_all(&state.pool)
    .await?;

    // this is a bit different from other projects/ endpoints - we want
    // to show history even for non-existing projects
    if project
        .as_ref()
        .is_none_or(|project| project.num_repos == 0)
        && cves.is_empty()
    {
        return nonexisting_project(&*state, ctx, project_name, project).await;
    }

    // sort by CVE number, then end version
    // XXX: this needs to include CPE files to be stable, as there may be multiple
    // records with equal CVE and version ranges, but different only by e.g. target_sw
    cves.sort_by(|a, b| {
        a.sort_key()
            .cmp(&b.sort_key())
            .then_with(|| {
                version_compare_nulls_last(&a.end_version.as_deref(), &b.end_version.as_deref())
            })
            .then_with(|| {
                version_compare_nulls_first(
                    &a.start_version.as_deref(),
                    &b.start_version.as_deref(),
                )
            })
    });

    let num_cves = cves.len();
    let mut aggregated_cves: IndexMap<CveAggregation, Vec<CveVersionRange>> = Default::default();

    for cve in cves {
        let range = CveVersionRange {
            text: cve.version_range_text(),
            highlighted: query
                .highlighted_version
                .as_ref()
                .map(|version| cve.is_version_in_range(version))
                .unwrap_or(false),
        };
        let aggregation = CveAggregation {
            cve_id: cve.cve_id,
            published: cve.published,
            last_modified: cve.last_modified,

            cpe_vendor: cve.cpe_vendor,
            cpe_product: cve.cpe_product,
            cpe_edition: cve.cpe_edition,
            cpe_lang: cve.cpe_lang,
            cpe_sw_edition: cve.cpe_sw_edition,
            cpe_target_sw: cve.cpe_target_sw,
            cpe_target_hw: cve.cpe_target_hw,
            cpe_other: cve.cpe_other,
        };
        aggregated_cves.entry(aggregation).or_default().push(range);
    }

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            project_name,
            project,
            num_cves,
            highlighted_version: query.highlighted_version.as_deref(),
            aggregated_cves,
        }
        .render()?,
    )
        .into_response())
}
