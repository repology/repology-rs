// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum::extract::{Form, Path, Query, State};
use axum::response::IntoResponse;
use axum_htmx::{HxRequest, VaryHxRequest};
use maud::html;
use tracing::info;

use crate::app::AppState;
use crate::components::report::{Report, ReportFlags, ReportUpdate};
use crate::components::reports::Reports;
use crate::html::render_html;
use crate::result::EndpointResult;

pub async fn handle_delete_report(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    Report::delete(&state.pool, id).await?;
    info!(id = id, "report deleted");
    Ok(html! {}.into_response())
}

pub async fn handle_patch_report(
    Path(id): Path<i32>,
    Query(flags): Query<ReportFlags>,
    State(state): State<Arc<AppState>>,
    Form(update): Form<ReportUpdate>,
) -> EndpointResult {
    Report::update(&state.pool, id, &update).await?;

    let report = Report::fetch(&state.pool, id).await?;

    info!(
        id = id,
        effname = report.effname,
        accepted = report.accepted,
        "report updated"
    );

    if flags.hide_processed && report.accepted.is_some() {
        Ok(html! {}.into_response())
    } else {
        Ok(report.render(&flags, &state.config).into_response())
    }
}

pub async fn handle_new_reports_page(
    HxRequest(hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let reports = Reports::fetch_new(&state.pool).await?;
    let content = html! {
        h1 .title { "New reports" };
        (reports.render(&ReportFlags{hide_processed: true}, &state.config))
    };
    let content = if hx_request {
        content
    } else {
        render_html(content)
    };
    Ok((VaryHxRequest {}, content).into_response())
}

pub async fn handle_all_reports_page(
    HxRequest(hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let reports = Reports::fetch_all(&state.pool).await?;
    let content = html! {
        h1 .title { "All reports" };
        (reports.render(&ReportFlags::default(), &state.config))
    };
    let content = if hx_request {
        content
    } else {
        render_html(content)
    };
    Ok((VaryHxRequest {}, content).into_response())
}
