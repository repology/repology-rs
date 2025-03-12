// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::State;
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use indoc::indoc;

use crate::endpoints::Endpoint;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn sitemap_index() -> EndpointResult {
    #[derive(Template)]
    #[template(path = "sitemaps/index.xml")]
    struct TemplateParams {
        ctx: TemplateContext,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::SitemapIndex);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        )],
        TemplateParams { ctx }.render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn sitemap_main() -> EndpointResult {
    #[derive(Template)]
    #[template(path = "sitemaps/main.xml")]
    struct TemplateParams {
        ctx: TemplateContext,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::SitemapMain);

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        )],
        TemplateParams { ctx }.render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn sitemap_repositories(State(state): State<Arc<AppState>>) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "sitemaps/repositories.xml")]
    struct TemplateParams {
        ctx: TemplateContext,
        repository_names: Vec<String>,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::SitemapRepositories);

    let repository_names = sqlx::query_scalar(indoc! {"
        SELECT name
        FROM repositories
        WHERE state = 'active'
        ORDER BY name
        LIMIT $1
    "})
    .bind(crate::constants::MAX_SITEMAP_ITEMS as i32)
    .fetch_all(&state.pool)
    .await?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        )],
        TemplateParams {
            ctx,
            repository_names,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn sitemap_maintainers(State(state): State<Arc<AppState>>) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "sitemaps/maintainers.xml")]
    struct TemplateParams {
        ctx: TemplateContext,
        maintainer_names: Vec<String>,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::SitemapMaintainers);

    let maintainer_names = sqlx::query_scalar(indoc! {"
        SELECT maintainer
        FROM maintainers
        ORDER BY num_projects DESC, maintainer
        LIMIT $1
    "})
    .bind(crate::constants::MAX_SITEMAP_ITEMS as i32)
    .fetch_all(&state.pool)
    .await?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        )],
        TemplateParams {
            ctx,
            maintainer_names,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn sitemap_projects(State(state): State<Arc<AppState>>) -> EndpointResult {
    #[derive(Template)]
    #[template(path = "sitemaps/projects.xml")]
    struct TemplateParams {
        ctx: TemplateContext,
        project_names: Vec<String>,
    }

    let ctx = TemplateContext::new_without_params(Endpoint::SitemapProjects);

    let project_names = sqlx::query_scalar(indoc! {"
        SELECT effname
        FROM metapackages
        -- the condition is tuned for metapackages_num_families_idx index to be used
        -- which guarantees us rather fast index-only scan
        WHERE num_repos_nonshadow > 0 AND num_families >= 5
        ORDER BY num_families DESC, effname
        LIMIT $1
    "})
    .bind(crate::constants::MAX_SITEMAP_ITEMS as i32)
    .fetch_all(&state.pool)
    .await?;

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        )],
        TemplateParams { ctx, project_names }.render()?,
    )
        .into_response())
}
