// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::handlers::problems::common::problems_generic;
use crate::result::HandlerResult;
use crate::routes::MyRoute;
use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(rename = "start")]
    pub start_project_name: Option<String>,
    #[serde(rename = "end")]
    pub end_project_name: Option<String>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all, fields(maintainer_name = maintainer_name, repository_name = repository_name, query = ?query)))]
pub async fn maintainer_problems(
    my_route: MyRoute,
    Path((maintainer_name, repository_name)): Path<(String, String)>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult {
    problems_generic(
        &my_route,
        &repository_name,
        Some(&maintainer_name),
        query.start_project_name.as_ref().map(|s| s.as_ref()),
        query.end_project_name.as_ref().map(|s| s.as_ref()),
        &state,
    )
    .await
}
