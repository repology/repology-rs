// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(iterator_try_collect)]
#![feature(coverage_attribute)]
#![feature(stmt_expr_attributes)]
#![feature(assert_matches)]
#![feature(duration_constructors)]

mod badges;
mod constants;
mod endpoints;
mod feeds;
mod font;
mod graphs;
mod package;
mod query;
mod repository_data;
mod result;
mod state;
mod static_files;
mod template_context;
mod url_for;
mod views;
mod xmlwriter;

use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Error};
use axum::{
    body::HttpBody,
    extract::{MatchedPath, Request},
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
    Router,
};
use metrics::{counter, histogram};
use sqlx::PgPool;
use tracing::{error, info, info_span, Instrument as _};

use crate::font::FontMeasurer;
use crate::repository_data::RepositoryDataCache;
use crate::state::AppState;
use crate::static_files::STATIC_FILES;

async fn track_metrics(matched_path: MatchedPath, req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();

    let path_for_metrics = {
        // normalize some paths which lead to the same endpoints; XXX this will hopefully be gone
        // someday when endpoints are redesigned (e.g. /projects/:bound/ → /projects/?start=)
        let mut path = matched_path
            .as_str()
            .trim_end_matches(":bound/")
            .trim_end_matches("/:sorting");
        if path.starts_with("/graph/total/") {
            path = "/graph/total/..."
        }
        if path.starts_with("/graph/repo/") {
            path = "/graph/repo/..."
        }

        path.to_owned()
    };

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    counter!("repology_webapp_http_requests_total", "path" => path_for_metrics.clone(), "status" => status)
        .increment(1);
    histogram!("repology_webapp_http_requests_duration_seconds", "path" => path_for_metrics.clone())
        .record(latency);

    if let Some(body_size) = response.body().size_hint().exact() {
        histogram!("repology_webapp_http_response_size_bytes", "path" => path_for_metrics)
            .record(body_size as f64);
    }

    response
}

#[cfg_attr(
    not(feature = "coverage"),
    tracing::instrument(name = "app init", skip_all)
)]
pub async fn create_app(pool: PgPool) -> Result<Router, Error> {
    info!("initializing font measurer");
    let font_measurer = FontMeasurer::new();

    info!("initializing repository data cache");
    let repository_data_cache = RepositoryDataCache::new(pool.clone())
        .await
        .context("error getting repository metadata")?;

    let state = Arc::new(AppState::new(pool, font_measurer, repository_data_cache));

    info!("initializing static files");
    let _ = &*STATIC_FILES;

    info!("launching background tasks");
    {
        let state = Arc::downgrade(&state);
        let task = async move {
            loop {
                tokio::time::sleep(crate::constants::REPOSITORY_CACHE_REFRESH_PERIOD).await;

                if let Some(state) = state.upgrade() {
                    state
                        .repository_data_cache
                        .update()
                        .await
                        .unwrap_or_else(|e| error!("repository data cache update failed {:?}", e));
                } else {
                    break;
                }
            }
        };
        tokio::task::spawn(task.instrument(info_span!(parent: None, "background task")));
    }

    info!("initializing routes");
    use crate::endpoints::Endpoint::*;
    #[rustfmt::skip]
    Ok(Router::new()
        .route(Api.path(), get(views::api_v1))
        .route(ApiV1.path(), get(views::api_v1))
        .route(ApiV1Project.path(), get(views::api_v1_project))
        .route(ApiV1Projects.path(), get(views::api_v1_projects))
        .route(ApiV1ProjectsBounded.path(), get(views::api_v1_projects_bounded))
        .route(BadgeLatestVersions.path(), get(views::badge_latest_versions))
        .route(BadgeTinyRepos.path(), get(views::badge_tiny_repos))
        .route(BadgeVersionForRepo.path(), get(views::badge_version_for_repo))
        .route(BadgeVerticalAllRepos.path(), get(views::badge_vertical_allrepos))
        .route(BadgeRepositoryBig.path(), get(views::badge_repository_big))
        .route(Docs.path(), get(views::docs))
        .route(DocsAbout.path(), get(views::docs_about))
        .route(DocsBots.path(), get(views::docs_bots))
        .route(DocsNotSupported.path(), get(views::docs_not_supported))
        .route(DocsRequirements.path(), get(views::docs_requirements))
        .route(Favicon.path(), get(views::favicon))
        .route(GraphTotalPackages.path(), get(views::graph_total_packages))
        .route(GraphTotalProjects.path(), get(views::graph_total_projects))
        .route(GraphTotalMaintainers.path(), get(views::graph_total_maintainers))
        .route(GraphTotalProblems.path(), get(views::graph_total_problems))
        .route(GraphRepoProblems.path(), get(views::graph_repository_problems))
        .route(GraphRepoMaintainers.path(), get(views::graph_repository_maintainers))
        .route(GraphRepoProjectsTotal.path(), get(views::graph_repository_projects_total))
        .route(GraphRepoProjectsUnique.path(), get(views::graph_repository_projects_unique))
        .route(GraphRepoProjectsNewest.path(), get(views::graph_repository_projects_newest))
        .route(GraphRepoProjectsOutdated.path(), get(views::graph_repository_projects_outdated))
        .route(GraphRepoProjectsProblematic.path(), get(views::graph_repository_projects_problematic))
        .route(GraphRepoProjectsVulnerable.path(), get(views::graph_repository_projects_vulnerable))
        .route(GraphRepoProjectsUniquePercent.path(), get(views::graph_repository_projects_unique_percent))
        .route(GraphRepoProjectsNewestPercent.path(), get(views::graph_repository_projects_newest_percent))
        .route(GraphRepoProjectsOutdatedPercent.path(), get(views::graph_repository_projects_outdated_percent))
        .route(GraphRepoProjectsProblematicPercent.path(), get(views::graph_repository_projects_problematic_percent))
        .route(GraphRepoProjectsVulnerablePercent.path(), get(views::graph_repository_projects_vulnerable_percent))
        .route(GraphRepoProjectsPerMaintainer.path(), get(views::graph_repository_projects_per_maintainer))
        .route(GraphRepoProblemsPer1000Projects.path(), get(views::graph_repository_problems_per_1000_projects))
        .route(Log.path(), get(views::log))
        .route(Maintainer.path(), get(views::maintainer))
        .route(MaintainerRepoFeed.path(), get(views::maintainer_repo_feed))
        .route(MaintainerRepoFeedAtom.path(), get(views::maintainer_repo_feed_atom))
        .route(MaintainerProblems.path(), get(views::maintainer_problems))
        .route(News.path(), get(views::news))
        .route(OpensearchMaintainer.path(), get(views::opensearch_maintainer))
        .route(OpensearchProject.path(), get(views::opensearch_project))
        .route(ProjectInformation.path(), get(views::project_information))
        .route(ProjectVersions.path(), get(views::project_versions))
        .route(ProjectPackages.path(), get(views::project_packages))
        .route(ProjectRelated.path(), get(views::project_related))
        .route(ProjectBadges.path(), get(views::project_badges))
        .route(Projects.path(), get(views::projects))
        .route(ProjectsBounded.path(), get(views::projects_bounded))
        .route(Repository.path(), get(views::repository))
        .route(RepositoryFeed.path(), get(views::repository_feed))
        .route(RepositoryFeedAtom.path(), get(views::repository_feed_atom))
        .route(RepositoryProblems.path(), get(views::repository_problems))
        .route(StaticFile.path(), get(views::static_file))
        .route(Tools.path(), get(views::tools))
        .route(LegacyBadgeVersionOnlyForRepo.path(), get(views::legacy_badge_version_only_for_repo))
        .route(LegacyProject.path(), get(views::legacy_metapackage_versions))
        .route(LegacyMetapackage.path(), get(views::legacy_metapackage_versions))
        .route(LegacyMetapackageVersions.path(), get(views::legacy_metapackage_versions))
        .route(LegacyMetapackagePackages.path(), get(views::legacy_metapackage_packages))
        .route(ToolProjectBy.path(), get(views::project_by))
        .route_layer(middleware::from_fn(track_metrics))
        .with_state(state))
}
