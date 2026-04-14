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

use crate::result::HandlerResult;
use crate::routes::MyRoute;
use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub autorefresh: bool,
}

#[derive(Template)]
#[template(path = "repositories/statistics.html")]
struct TemplateParams<'a> {
    my_route: &'a MyRoute,
    sorting: &'a str,
    total_projects: i32,
    total_packages: i32,
    repositories: Vec<Repository>,
    autorefresh: bool,
}

#[derive(FromRow)]
struct Repository {
    name: String,
    title: String,

    num_projects: i32,
    num_projects_unique: i32,
    num_projects_newest: i32,
    num_projects_outdated: i32,
    num_projects_comparable: i32,
    num_projects_problematic: i32,
    num_projects_vulnerable: i32,

    num_maintainers: i32,
    num_problems: i32,

    is_shadow: bool,
}

pub async fn repositories_statistics_generic(
    my_route: &MyRoute,
    sorting: &str,
    query: QueryParams,
    state: &AppState,
) -> HandlerResult {
    let (order, sorting) = match sorting {
        "newest" => ("num_metapackages_newest DESC, sortname", sorting),
        "pnewest" => (
            "num_metapackages_newest::real / nullif(num_metapackages_comparable, 0) DESC NULLS LAST, sortname",
            sorting,
        ),
        "outdated" => ("num_metapackages_outdated DESC, sortname", sorting),
        "poutdated" => (
            "num_metapackages_outdated::real / nullif(num_metapackages_comparable, 0) DESC NULLS LAST, sortname",
            sorting,
        ),
        "total" => ("num_metapackages DESC, sortname", sorting),
        "nonunique" => (
            "num_metapackages - num_metapackages_unique DESC, sortname",
            sorting,
        ),
        "vulnerable" => ("num_metapackages_vulnerable DESC, sortname", sorting),
        "pvulnerable" => (
            "num_metapackages_vulnerable::real / nullif(num_metapackages, 0) DESC NULLS LAST, sortname",
            sorting,
        ),
        _ => ("sortname", "name"),
    };

    let repositories: Vec<Repository> = sqlx::query_as(&format!(
        indoc! {r#"
            SELECT
                name,
                "desc" AS title,
                num_metapackages AS num_projects,
                num_metapackages_unique AS num_projects_unique,
                num_metapackages_newest AS num_projects_newest,
                num_metapackages_outdated AS num_projects_outdated,
                num_metapackages_comparable AS num_projects_comparable,
                num_metapackages_problematic AS num_projects_problematic,
                num_metapackages_vulnerable AS num_projects_vulnerable,
                num_maintainers,
                num_problems,
                coalesce((metadata->'shadow')::boolean, false) AS is_shadow
            FROM repositories
            WHERE state = 'active'
            ORDER BY {}
        "#},
        order
    ))
    .fetch_all(&state.pool)
    .await?;

    let (total_projects, total_packages): (i32, i32) =
        sqlx::query_as("SELECT num_metapackages AS num_projects, num_packages FROM statistics")
            .fetch_one(&state.pool)
            .await?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            my_route,
            sorting,
            total_projects,
            total_packages,
            repositories,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(
    not(feature = "coverage"),
    tracing::instrument(skip_all, fields(query = ?query))
)]
pub async fn repositories_statistics_default(
    my_route: MyRoute,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult {
    repositories_statistics_generic(&my_route, "name", query, &state).await
}

#[cfg_attr(
    not(feature = "coverage"),
    tracing::instrument(skip_all, fields(query = ?query))
)]
pub async fn repositories_statistics_sorted(
    my_route: MyRoute,
    Path(sorting): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult {
    repositories_statistics_generic(&my_route, &sorting, query, &state).await
}
