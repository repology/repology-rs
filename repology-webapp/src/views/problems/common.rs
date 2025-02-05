// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

// triggered inside template
#![allow(clippy::manual_range_contains)]

use askama::Template;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use indoc::indoc;
use sqlx::FromRow;

use crate::repository_data::RepositoryData;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

#[derive(Debug, FromRow)]
pub struct Problem {
    effname: String,
    url: Option<String>,
    visiblename: String,
    maintainer: Option<String>,
    data: serde_json::Value,
    kind: String,
}

#[derive(PartialEq)]
struct KeyRange<'a> {
    first: &'a str,
    last: &'a str,
}

struct Pagination<'a> {
    whole_range: KeyRange<'a>,
    page_range: Option<KeyRange<'a>>,
}

impl Pagination<'_> {
    fn is_first_page(&self) -> bool {
        self.page_range
            .as_ref()
            .is_some_and(|range| range.first == self.whole_range.first)
    }

    fn is_last_page(&self) -> bool {
        self.page_range
            .as_ref()
            .is_some_and(|range| range.last == self.whole_range.last)
    }
}

#[derive(Template)]
#[template(path = "problems/index.html")]
struct TemplateParams<'a> {
    ctx: TemplateContext,
    maintainer_name: Option<&'a str>,
    problems: Vec<Problem>,
    repository_data: &'a RepositoryData,
    pagination: Option<Pagination<'a>>,
}

pub async fn problems_generic(
    ctx: TemplateContext,
    repository_name: &str,
    maintainer_name: Option<&str>,
    start_project_name: Option<&str>,
    end_project_name: Option<&str>,
    state: &AppState,
) -> EndpointResult {
    let repositories_data = state.repository_data_cache.snapshot();

    let Some(repository_data) = repositories_data.active_repository(repository_name) else {
        return Ok((StatusCode::NOT_FOUND, "repository not found".to_owned()).into_response());
    };

    let problems: Vec<Problem> = sqlx::query_as(indoc! {r#"
        WITH unsorted AS (
            SELECT
                packages.*,  -- XXX: remove this after #1129
                maintainer,
                "type"::text AS kind,
                data,
                (
                    SELECT url
                    FROM links
                    WHERE id IN (
                        SELECT
                            link_id
                        FROM (
                            SELECT
                                (json_array_elements(links)->>0)::integer AS link_type,
                                (json_array_elements(links)->>1)::integer AS link_id
                        ) AS expanded_links
                        WHERE
                            link_type IN (
                                5, -- PACKAGE_HOMEPAGE
                                7, -- PACKAGE_SOURCES
                                9, -- PACKAGE_RECIPE
                                10 -- PACKAGE_RECIPE_RAW
                            )
                    ) --AND coalesce(ipv4_success, true)  -- XXX: better display link status
                    LIMIT 1
                ) AS url
            FROM problems
            INNER JOIN packages ON packages.id = problems.package_id
            WHERE
                problems.repo = $1
                AND ($2 IS NULL OR maintainer = $2)
                AND ($3 IS NULL OR problems.effname >= $3)
                AND ($4 IS NULL OR problems.effname <= $4)
            ORDER BY
                CASE WHEN $4 IS NULL THEN problems.effname END,
                CASE WHEN $4 IS NOT NULL THEN problems.effname END DESC,
                CASE WHEN $4 IS NULL THEN problems.maintainer END,
                CASE WHEN $4 IS NOT NULL THEN problems.maintainer END DESC
            LIMIT $5
        )
        SELECT *
        FROM unsorted
        ORDER BY effname, maintainer;
    "#})
    .bind(repository_name)
    .bind(maintainer_name)
    .bind(start_project_name)
    .bind(end_project_name)
    .bind(crate::constants::PROBLEMS_PER_PAGE as i32)
    .fetch_all(&state.pool)
    .await?;

    let page_range = problems
        .first()
        .zip(problems.last())
        .map(|(first, last)| (first.effname.clone(), last.effname.clone()));
    let page_range = page_range
        .as_ref()
        .map(|(first, last)| KeyRange { first, last });

    let range: (Option<String>, Option<String>) = sqlx::query_as(indoc! {r#"
        SELECT
            min(effname),
            max(effname)
        FROM problems
        WHERE
            repo = $1
            AND ($2 IS NULL OR maintainer = $2)
    "#})
    .bind(repository_name)
    .bind(maintainer_name)
    .fetch_one(&state.pool)
    .await?;

    let whole_range = range
        .0
        .as_ref()
        .zip(range.1.as_ref())
        .map(|(first, last)| KeyRange { first, last });

    let pagination = whole_range
        .map(move |whole_range| Pagination {
            whole_range,
            page_range,
        })
        .filter(|pagination| {
            pagination
                .page_range
                .as_ref()
                .is_none_or(|page_range| *page_range != pagination.whole_range)
        });

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            maintainer_name,
            problems,
            repository_data,
            pagination,
        }
        .render()?,
    )
        .into_response())
}
