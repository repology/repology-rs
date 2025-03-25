// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use serde::Deserialize;
use sqlx::FromRow;

use crate::endpoints::Endpoint;
use crate::repository_data::RepositoriesDataSnapshot;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    pub search: String,
}

struct MaintainerPerRepositoryCounters {
    #[expect(unused)]
    num_packages: i32,
    num_projects: i32,
    num_projects_newest: i32,
    num_projects_outdated: i32,
    num_projects_problematic: i32,
    #[expect(unused)]
    num_projects_vulnerable: i32,
}

impl<'de> Deserialize<'de> for MaintainerPerRepositoryCounters {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = MaintainerPerRepositoryCounters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an array of 6 integers")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<MaintainerPerRepositoryCounters, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let num_packages = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let num_projects = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let num_projects_newest = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                let num_projects_outdated = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;
                let num_projects_problematic = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(4, &self))?;
                // old entries may miss vulnerable projects counter
                let num_projects_vulnerable = seq.next_element()?.unwrap_or_default();
                Ok(MaintainerPerRepositoryCounters {
                    num_packages,
                    num_projects,
                    num_projects_newest,
                    num_projects_outdated,
                    num_projects_problematic,
                    num_projects_vulnerable,
                })
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

#[derive(FromRow)]
struct DbMaintainer {
    name: String,
    num_projects: i32,
    num_repositories: i32,
    first_seen: DateTime<Utc>,
    per_repository_counters: sqlx::types::Json<HashMap<String, MaintainerPerRepositoryCounters>>,
}

#[derive(FromRow)]
struct Maintainer {
    name: String,
    num_projects: i32,
    num_repositories: i32,
    first_seen: DateTime<Utc>,
    best_repository_name: String,
    best_repository_counters: MaintainerPerRepositoryCounters,
}

#[derive(Template)]
#[template(path = "maintainers/index.html")]
struct TemplateParams<'a> {
    ctx: TemplateContext,
    query: QueryParams,
    repositories_data: &'a RepositoriesDataSnapshot,
    maintainers: Vec<Maintainer>,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
async fn maintainers_generic(
    ctx: TemplateContext,
    start_maintainer_name: Option<&str>,
    end_maintainer_name: Option<&str>,
    query: QueryParams,
    state: &AppState,
) -> EndpointResult {
    let maintainers: Vec<DbMaintainer> = sqlx::query_as(indoc! {"
        SELECT
            *
        FROM (
            SELECT
                maintainer AS name,
                num_projects,
                num_repos AS num_repositories,
                first_seen,
                counts_per_repo AS per_repository_counters
            FROM maintainers
            WHERE
                num_packages > 0
                AND ($1 IS NULL OR maintainer >= $1)
                AND ($2 IS NULL OR maintainer <= $2)
                AND ($3 IS NULL OR maintainer LIKE ('%' || $3 || '%'))
            ORDER BY
                CASE WHEN $2 IS NULL THEN maintainer ELSE NULL END,
                CASE WHEN $2 IS NOT NULL THEN maintainer ELSE NULL END DESC
            LIMIT $4
        ) AS tmp
        ORDER BY name
    "})
    .bind(start_maintainer_name)
    .bind(end_maintainer_name)
    .bind(Some(&query.search).filter(|search| !search.is_empty()))
    .bind(crate::constants::MAINTAINERS_PER_PAGE as i32)
    .fetch_all(&state.pool)
    .await?;

    let repositories_data = state.repository_data_cache.snapshot();

    let maintainers = maintainers
        .into_iter()
        .filter_map(|maintainer| {
            let best_repository = maintainer.per_repository_counters.0.into_iter().max_by(
                |(repository_name_a, repository_counters_a),
                 (repository_name_b, repository_counters_b)| {
                    repository_counters_a
                        .num_projects_newest
                        .cmp(&repository_counters_b.num_projects_newest)
                        .then_with(|| {
                            repository_counters_a
                                .num_projects_outdated
                                .cmp(&repository_counters_b.num_projects_outdated)
                                .reverse()
                        })
                        .then_with(|| {
                            let num_projects_newest_a = repositories_data
                                .repository(repository_name_a)
                                .map(|repository| repository.num_projects_newest)
                                .unwrap_or(0);
                            let num_projects_newest_b = repositories_data
                                .repository(repository_name_b)
                                .map(|repository| repository.num_projects_newest)
                                .unwrap_or(0);

                            num_projects_newest_a.cmp(&num_projects_newest_b)
                        })
                        .then_with(|| repository_name_a.cmp(repository_name_b))
                },
            )?;

            Some(Maintainer {
                name: maintainer.name,
                num_projects: maintainer.num_projects,
                num_repositories: maintainer.num_repositories,
                first_seen: maintainer.first_seen,
                best_repository_name: best_repository.0,
                best_repository_counters: best_repository.1,
            })
        })
        .collect();

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            query,
            repositories_data: &repositories_data,
            maintainers,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all, fields(query = ?query)))]
pub async fn maintainers(
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new_without_params(Endpoint::Maintainers);

    maintainers_generic(ctx, None, None, query, &state).await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all, fields(bound = bound, query = ?query)))]
pub async fn maintainers_bounded(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
    Path(bound): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::MaintainersBounded, gen_path, gen_query);

    if let Some(end) = bound.strip_prefix("..") {
        maintainers_generic(ctx, None, Some(end), query, &state).await
    } else {
        maintainers_generic(ctx, Some(&bound), None, query, &state).await
    }
}
