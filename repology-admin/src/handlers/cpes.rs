// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum::extract::{Form, Path, Query, State};
use axum::response::IntoResponse;
use axum_htmx::{HxRequest, VaryHxRequest};
use maud::html;
use serde::Deserialize;
use tracing::info;

use crate::app::AppState;
use crate::components::cpe::Cpe;
use crate::components::cpe_item::CpeItem;
use crate::components::cpe_items::CpeItems;
use crate::html::render_html;
use crate::result::EndpointResult;

#[derive(Deserialize)]
pub struct GetCpesParams {
    search: String,
}

pub async fn handle_cpes(
    Query(query): Query<GetCpesParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    Ok(CpeItems::fetch(&state.pool, &query.search)
        .await?
        .render(&state.config)
        .into_response())
}

pub async fn handle_cpe_form(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    Ok(CpeItem::fetch(&state.pool, id)
        .await?
        .render_form()
        .into_response())
}

pub async fn handle_cpe(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> EndpointResult {
    Ok(CpeItem::fetch(&state.pool, id)
        .await?
        .render(&state.config)
        .into_response())
}

pub async fn handle_new_cpe_form() -> EndpointResult {
    Ok(Cpe::new_for_create().render_new_form(false).into_response())
}

pub async fn handle_new_cpe(
    State(state): State<Arc<AppState>>,
    Form(cpe): Form<Cpe>,
) -> EndpointResult {
    if cpe.is_valid() {
        let id = cpe.create(&state.pool).await?;
        let cpe_item = CpeItem::fetch(&state.pool, id).await?;
        info!(id = id, effname = cpe.effname, "CPE created");
        Ok(cpe_item.render(&state.config).into_response())
    } else {
        Ok(cpe.render_new_form(true).into_response())
    }
}

pub async fn handle_update_cpe(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>,
    Form(cpe): Form<Cpe>,
) -> EndpointResult {
    if cpe.is_valid() {
        cpe.update(&state.pool, id).await?;
        let cpe_item = CpeItem::fetch(&state.pool, id).await?;
        info!(id = id, effname = cpe.effname, "CPE updated");
        Ok(cpe_item.render(&state.config).into_response())
    } else {
        let mut cpe_item = CpeItem::fetch(&state.pool, id).await?;
        cpe_item.cpe = cpe;
        Ok(cpe_item.render_form().into_response())
    }
}

pub async fn handle_delete_cpe(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    CpeItem::delete(&state.pool, id).await?;
    info!(id = id, "CPE deleted");
    Ok(html! {}.into_response())
}

pub async fn handle_cpes_page(HxRequest(hx_request): HxRequest) -> EndpointResult {
    let content = html! {
        h1 .title { "CPEs" }
        .field .is-grouped {
            .control .is-expanded {
                input
                    .input
                    type="search"
                    name="search"
                    placeholder="Type project, CPE vendor, or CPE product substring"
                    hx-get="/parts/cpes"
                    hx-trigger="input changed delay:500ms, keyup[key=='Enter'], load"
                    hx-target="#search-results";
            }
            .control {
                button .button .is-success hx-get="/parts/cpes/form" hx-target="#search-results" hx-swap="afterbegin" {
                    "Add new"
                }
            }
        }
        table .table .is-striped .is-fullwidth {
            thead {
                tr {
                    td { "Project" };
                    td { "CPE" };
                    td .has-text-centered { abbr title="Project status" { "Proj" } };
                    td .has-text-centered { abbr title="CVEs are present" { "CVEs" }  };
                    td .has-text-centered { abbr title="CPE dictionary entry is present" { "Dict" } };
                    td .has-text-centered { "Actions" };
                }
            }
            tbody #search-results {
                tr {
                    td colspan="6" .has-text-centered {
                        "No results"
                    }
                }
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
