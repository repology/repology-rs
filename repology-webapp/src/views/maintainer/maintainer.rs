// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use indoc::indoc;
use itertools::Itertools;
use metrics::histogram;
use sqlx::FromRow;

use crate::endpoints::Endpoint;
use crate::repository_data::RepositoriesDataSnapshot;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

#[derive(FromRow)]
struct Maintainer {
    num_packages: i32,

    num_projects: i32,
    num_projects_newest: i32,
    num_projects_outdated: i32,
    num_projects_problematic: i32,
    num_projects_vulnerable: i32,

    counts_per_repo: sqlx::types::Json<HashMap<String, Vec<i32>>>,

    num_projects_per_category: sqlx::types::Json<HashMap<String, i32>>,

    orphaned_at: Option<DateTime<Utc>>,
}

fn generate_maintainer_links(maintainer_name: &str) -> Vec<String> {
    let mut res = vec![];
    if let Some((name, domain)) = maintainer_name.split_once('@') {
        match domain {
            "cpan" => res.push(format!("https://metacpan.org/author/{}", name)),
            "aur" => res.push(format!("https://aur.archlinux.org/account/{}", name)),
            "altlinux.org" | "altlinux.ru" => {
                res.push(format!("https://sisyphus.ru/en/packager/{}", name))
            }
            "github" => res.push(format!("https://github.com/{}", name)),
            "freshcode" => res.push(format!("https://freshcode.club/search?user={}", name)),
            _ => {}
        }
        if domain.contains('.') {
            res.push(format!("mailto:{}", maintainer_name))
        }
    }
    res
}

// TODO:
// This endpoint (and maybe other similar ones) should be refactored the following way:
// - All code which transforms data from database objects into template objects should
//   be moved to template object constructors
// - There should be an object for maintainer counters too, so we don't need to have
//   partially deconstructed Maintainer, and we also could fill percentages right away
//   instead of trying to format these in templates

struct MaintainerRepository {
    pub name: String,
    pub num_packages: usize,
    pub num_projects: usize,
    pub num_projects_newest: usize,
    pub num_projects_outdated: usize,
    pub num_projects_problematic: usize,
    pub num_projects_vulnerable: usize,
}

struct MaintainerCategory {
    pub name: String,
    pub num_projects: usize, // XXX: is this really projects, not packages?
}

#[derive(FromRow)]
struct SimilarMaintainer {
    pub name: String,
    pub num_common_projects: i32,
    pub score: f32,
}

#[derive(Template)]
#[template(path = "maintainer/unknown.html")]
struct TemplateParamsUnknown<'a> {
    ctx: TemplateContext,
    maintainer_name: &'a str,
}

#[derive(Template)]
#[template(path = "maintainer/gone.html")]
struct TemplateParamsGone<'a> {
    ctx: TemplateContext,
    maintainer_name: &'a str,
    maintainer: Maintainer,
}

#[derive(Template)]
#[template(path = "maintainer.html")]
struct TemplateParams<'a> {
    ctx: TemplateContext,
    maintainer_name: &'a str,
    maintainer: Maintainer,
    maintainer_categories: Vec<MaintainerCategory>,
    maintainer_repositories: Vec<MaintainerRepository>,
    similar_maintainers_columns: [&'a [SimilarMaintainer]; 2],
    projects: Vec<String>,
    is_fallback_maintainer: bool,
    maintainer_links: Vec<String>,
    repositories_data: &'a RepositoriesDataSnapshot,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn maintainer(
    Path(maintainer_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new_without_params(Endpoint::Maintainer);

    let maintainer_name = maintainer_name.to_lowercase();

    let maintainer: Option<Maintainer> = sqlx::query_as(indoc! {"
        SELECT
            num_packages,
            num_projects,
            num_projects_newest,
            num_projects_outdated,
            num_projects_problematic,
            num_projects_vulnerable,
            coalesce(counts_per_repo, '{}'::jsonb) AS counts_per_repo,
            coalesce(num_projects_per_category, '{}'::jsonb) AS num_projects_per_category,
            orphaned_at
        FROM maintainers
        WHERE maintainer = $1
    "})
    .bind(&maintainer_name)
    .fetch_optional(&state.pool)
    .await?;

    let mut maintainer = match maintainer {
        Some(maintainer) if maintainer.num_packages > 0 => maintainer,
        Some(maintainer) => {
            return Ok((
                StatusCode::NOT_FOUND, // or should it be GONE?
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
                )],
                TemplateParamsGone {
                    ctx,
                    maintainer_name: &maintainer_name,
                    maintainer,
                }
                .render()?,
            )
                .into_response());
        }
        None => {
            return Ok((
                StatusCode::NOT_FOUND,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
                )],
                TemplateParamsUnknown {
                    ctx,
                    maintainer_name: &maintainer_name,
                }
                .render()?,
            )
                .into_response());
        }
    };

    let maintainer_categories: Vec<_> = std::mem::take(&mut maintainer.num_projects_per_category.0)
        .into_iter()
        .map(|(name, num_projects)| MaintainerCategory {
            name,
            num_projects: num_projects as usize,
        })
        .sorted_by(|a, b| {
            a.num_projects
                .cmp(&b.num_projects)
                .reverse()
                .then_with(|| a.name.cmp(&b.name))
        })
        .collect();

    let maintainer_repositories: Vec<_> = std::mem::take(&mut maintainer.counts_per_repo.0)
        .into_iter()
        .map(|(name, fields)| {
            // there may be 5 or 6 fields - vulnerable projects count was added at some
            // point and there are still maintainers which do not have it filled
            MaintainerRepository {
                name,
                num_packages: fields[0] as usize,
                num_projects: fields[1] as usize,
                num_projects_newest: fields[2] as usize,
                num_projects_outdated: fields[3] as usize,
                num_projects_problematic: fields[4] as usize,
                num_projects_vulnerable: *fields.get(5).unwrap_or(&0) as usize,
            }
        })
        .sorted_by(|a, b| {
            a.num_projects_newest
                .cmp(&b.num_projects_newest)
                .reverse()
                .then_with(|| a.num_projects_outdated.cmp(&b.num_projects_outdated))
        })
        .collect();

    let all_queries_start = Instant::now();

    let projects_query_start = Instant::now();
    let projects: Vec<String> = sqlx::query_scalar(indoc! {"
        SELECT
            effname
        FROM maintainer_metapackages
        WHERE maintainer_id = (SELECT id FROM maintainers WHERE maintainer = $1)
        ORDER BY effname
        LIMIT $2
    "})
    .bind(&maintainer_name)
    .bind(crate::constants::MAX_MAINTAINER_PROJECTS as i32)
    .fetch_all(&state.pool)
    .await?;
    let projects_query_duration = projects_query_start.elapsed();

    let similar_maintainers_query_start = Instant::now();
    // this obscure request needs some clarification
    //
    // what we calculate as score here is actually Jaccard index
    // (see wikipedia) for two sets (of projects maintained by
    // two maintainers)
    //
    // let M = set of projects for maintainer passed to this function
    // let C = set of projects for other maintainer we test for similarity
    //
    // score = |M⋂C| / |M⋃C| = |M⋂C| / (|M| + |C| - |M⋂C|)
    //
    // - num_projects_common is |M⋂C|
    // - num_projects is |C|
    // - sub-select just gets |M|
    // - the divisor thus is |M⋃C| = |M| + |C| - |M⋂C|
    let similar_maintainers: Vec<SimilarMaintainer> = sqlx::query_as(indoc! {"
        SELECT
            maintainer AS name,
            num_common_projects::integer,
            100.0::real * num_common_projects::real / (
                num_projects - num_common_projects + (
                    SELECT num_projects
                    FROM maintainers
                    WHERE maintainer = $1
                )
            )::real AS score
        FROM
            (
                SELECT
                    maintainer_id,
                    count(*) AS num_common_projects
                FROM
                    maintainer_metapackages
                WHERE
                    maintainer_id != (SELECT id FROM maintainers WHERE maintainer = $1) AND
                    effname IN (
                        SELECT
                            effname
                        FROM maintainer_metapackages
                        WHERE maintainer_id = (SELECT id FROM maintainers WHERE maintainer = $1)
                    )
                GROUP BY maintainer_id
            ) AS intersecting_counts
            INNER JOIN maintainers ON(maintainer_id = maintainers.id)
        ORDER BY score DESC
        LIMIT $2
    "})
    .bind(&maintainer_name)
    .bind(crate::constants::NUM_SIMILAR_MAINTAINERS as i32)
    .fetch_all(&state.pool)
    .await?;
    let similar_maintainers_query_duration = similar_maintainers_query_start.elapsed();

    let all_queries_duration = all_queries_start.elapsed();

    histogram!("repology_webapp_maintainer_query_duration_seconds", "type" => "projects")
        .record(projects_query_duration.as_secs_f64());
    histogram!("repology_webapp_maintainer_query_duration_seconds", "type" => "similar_maintainers")
        .record(similar_maintainers_query_duration.as_secs_f64());
    histogram!("repology_webapp_maintainer_query_duration_seconds", "type" => "all")
        .record(all_queries_duration.as_secs_f64());

    let similar_maintainers_columns =
        similar_maintainers.split_at(similar_maintainers.len().div_ceil(2));

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            maintainer_name: &maintainer_name,
            maintainer,
            maintainer_categories,
            maintainer_repositories,
            similar_maintainers_columns: [
                similar_maintainers_columns.0,
                similar_maintainers_columns.1,
            ],
            projects,
            is_fallback_maintainer: maintainer_name.starts_with("fallback-mnt-")
                && maintainer_name.ends_with("@repology"),
            maintainer_links: generate_maintainer_links(&maintainer_name),
            repositories_data: &state.repository_data_cache.snapshot(),
        }
        .render()?,
    )
        .into_response())
}
