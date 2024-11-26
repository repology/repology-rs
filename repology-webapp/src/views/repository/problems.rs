// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::endpoints::Endpoint;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;
use crate::views::problems::common::problems_generic;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(rename = "start")]
    pub start_project_name: Option<String>,
    #[serde(rename = "end")]
    pub end_project_name: Option<String>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn repository_problems(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
    Path(repository_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::RepositoryProblems, gen_path, gen_query);

    problems_generic(
        ctx,
        &repository_name,
        None,
        query.start_project_name.as_ref().map(|s| s.as_ref()),
        query.end_project_name.as_ref().map(|s| s.as_ref()),
        &*state,
    )
    .await
}
