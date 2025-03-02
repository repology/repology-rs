// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
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
#[template(path = "repositories/packages.html")]
struct TemplateParams {
    ctx: TemplateContext,
    total_packages: i32,
    repositories: Vec<Repository>,
    autorefresh: bool,
}

#[derive(FromRow)]
struct Repository {
    name: String,
    title: String,

    num_packages: i32,
    num_packages_newest: i32,
    num_packages_devel: i32,
    num_packages_unique: i32,
    num_packages_outdated: i32,
    num_packages_legacy: i32,
    num_packages_rolling: i32,
    num_packages_noscheme: i32,
    num_packages_incorrect: i32,
    num_packages_untrusted: i32,
    num_packages_ignored: i32,
    num_packages_vulnerable: i32,
}

#[cfg_attr(
    not(feature = "coverage"),
    tracing::instrument(skip(gen_path, gen_query, state))
)]
pub async fn repositories_packages(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::RepositoriesPackages, gen_path, gen_query);

    let repositories: Vec<Repository> = sqlx::query_as(indoc! {r#"
        SELECT
            name,
            "desc" AS title,
            num_packages,
            num_packages_newest,
            num_packages_devel,
            num_packages_unique,
            num_packages_outdated,
            num_packages_legacy,
            num_packages_rolling,
            num_packages_noscheme,
            num_packages_incorrect,
            num_packages_untrusted,
            num_packages_ignored,
            num_packages_vulnerable
        FROM repositories
        WHERE state = 'active'
        ORDER BY sortname
    "#})
    .fetch_all(&state.pool)
    .await?;

    let total_packages = sqlx::query_scalar("SELECT num_packages FROM statistics")
        .fetch_one(&state.pool)
        .await?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            total_packages,
            repositories,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
