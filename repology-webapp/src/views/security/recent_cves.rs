// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use serde::Deserialize;
use sqlx::FromRow;

use crate::endpoints::Endpoint;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub autorefresh: bool,
}

#[derive(Template)]
#[template(path = "security/recent-cves.html")]
struct TemplateParams {
    ctx: TemplateContext,
    cves: Vec<Cve>,
    autorefresh: bool,
}

#[derive(FromRow)]
struct Cve {
    pub published: DateTime<Utc>,
    pub cve_id: String,
    pub is_recent: bool,
    pub project_names: Vec<String>,
}

#[cfg_attr(
    not(feature = "coverage"),
    tracing::instrument(skip(gen_path, gen_query, state))
)]
pub async fn recent_cves(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::SecurityRecentCves, gen_path, gen_query);

    let cves: Vec<Cve> = sqlx::query_as(indoc! {r#"
        WITH expanded_matches AS (
            SELECT DISTINCT
                cve_id,
                published,

                jsonb_array_elements(matches)->>0 AS cpe_vendor,
                jsonb_array_elements(matches)->>1 AS cpe_product,
                jsonb_array_elements(matches)->>2 AS cpe_edition,
                jsonb_array_elements(matches)->>3 AS cpe_lang,
                jsonb_array_elements(matches)->>4 AS cpe_sw_edition,
                jsonb_array_elements(matches)->>5 AS cpe_target_sw,
                jsonb_array_elements(matches)->>6 AS cpe_target_hw,
                jsonb_array_elements(matches)->>7 AS cpe_other
            FROM cves
            WHERE published > now() - $2
        )
        SELECT
            published,
            cve_id,
            manual_cpes.added_ts > now() - interval '7 day' AS is_recent,
            array_agg(DISTINCT effname ORDER BY effname) AS project_names
        FROM expanded_matches INNER JOIN manual_cpes ON
            expanded_matches.cpe_product = manual_cpes.cpe_product AND
            expanded_matches.cpe_vendor = manual_cpes.cpe_vendor AND
            coalesce(nullif(expanded_matches.cpe_edition, '*') = nullif(manual_cpes.cpe_edition, '*'), TRUE) AND
            coalesce(nullif(expanded_matches.cpe_lang, '*') = nullif(manual_cpes.cpe_lang, '*'), TRUE) AND
            coalesce(nullif(expanded_matches.cpe_sw_edition, '*') = nullif(manual_cpes.cpe_sw_edition, '*'), TRUE) AND
            coalesce(nullif(expanded_matches.cpe_target_sw, '*') = nullif(manual_cpes.cpe_target_sw, '*'), TRUE) AND
            coalesce(nullif(expanded_matches.cpe_target_hw, '*') = nullif(manual_cpes.cpe_target_hw, '*'), TRUE) AND
            coalesce(nullif(expanded_matches.cpe_other, '*') = nullif(manual_cpes.cpe_other, '*'), TRUE)
        GROUP BY cve_id, published, is_recent
        ORDER BY published DESC, cve_id DESC
        LIMIT $1
    "#})
    .bind(&(crate::constants::RECENT_CVES_MAX_COUNT as i32))
    .bind(&crate::constants::RECENT_CVES_MAX_AGE)
    .fetch_all(&state.pool)
    .await?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            cves,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
