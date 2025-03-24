// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum_htmx::{HxRequest, VaryHxRequest};
use chrono::{DateTime, Utc};
use maud::html;
use serde::Deserialize;

use crate::app::AppState;
use crate::components::cves::Cves;
use crate::html::render_html;
use crate::result::EndpointResult;

#[derive(Deserialize)]
pub struct CvesQuery {
    before: DateTime<Utc>,
}

pub async fn handle_cves(
    Query(query): Query<CvesQuery>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let cves = Cves::fetch(&state.pool, query.before).await?;

    Ok(cves.render(&state.config).into_response())
}

pub async fn handle_cves_page(
    HxRequest(hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let cves = Cves::fetch(&state.pool, Utc::now()).await?;

    let content = html! {
        h1 .title { "CVEs" }
        table .table .is-striped .is-fullwidth {
            thead {
                tr {
                    td { "Last Modified" }
                    td { "CPE" }
                    td { "CVEs" }
                    td { "Candidates" }
                }
            }
            tbody {
                (cves.render(&state.config))
            }
        }
    };
    let content = if hx_request {
        content
    } else {
        render_html(content)
    };
    Ok((VaryHxRequest {}, content).into_response())
}
