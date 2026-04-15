// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum_myroutes::routes;

use crate::handlers;
use crate::state::AppState;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    #[default]
    None,

    Docs,
    Maintainers,
    News,
    Projects,
    Repositories,
    Security,
    Tools,
}

#[derive(Default, Clone, Copy)]
pub struct RouteProps {
    pub section: Section,
    pub allow_embedding: bool,
}

// route ordering:
// static -> index -> pages according to navbar -> supplementary pages -> supplementary routes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[routes(props_type = RouteProps, state_type = Arc<AppState>)]
pub enum Route {
    // Static
    #[get("/static/{file_name}", handler = handlers::static_file)]
    StaticFile,

    // Index
    #[get("/", handler = handlers::index)]
    Index,

    // Projects
    #[get("/projects/", handler = handlers::projects, props = RouteProps { section: Section::Projects, ..Default::default() })]
    Projects,
    #[get("/projects/{bound}/", handler = handlers::projects_bounded, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectsBounded,

    // Project
    #[get("/project/{project_name}/versions", handler = handlers::project_versions, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectVersions,
    #[get("/project/{project_name}/versions-compact", handler = handlers::project_versions_compact, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectVersionsCompact,
    #[get("/project/{project_name}/packages", handler = handlers::project_packages, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectPackages,
    #[get("/project/{project_name}/information", handler = handlers::project_information, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectInformation,
    #[get("/project/{project_name}/history", handler = handlers::project_history, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectHistory,
    #[get("/project/{project_name}/related", handler = handlers::project_related, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectRelated,
    #[get("/project/{project_name}/badges", handler = handlers::project_badges, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectBadges,
    #[get("/project/{project_name}/report", handler = handlers::project_report_get, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectReport,
    #[post("/project/{project_name}/report", handler = handlers::project_report_post, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectReportPost,
    #[get("/project/{project_name}/cves", handler = handlers::project_cves, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectCves,

    // Maintainers
    #[get("/maintainers/", handler = handlers::maintainers, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    Maintainers,
    #[get("/maintainers/{bound}/", handler = handlers::maintainers_bounded, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    MaintainersBounded,

    // Maintainer
    #[get("/maintainer/{maintainer_name}", handler = handlers::maintainer, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    Maintainer,
    #[get("/maintainer/{maintainer_name}/feed-for-repo/{repository_name}", handler = handlers::maintainer_repo_feed, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    MaintainerRepoFeed,
    #[get("/maintainer/{maintainer_name}/feed-for-repo/{repository_name}/atom", handler = handlers::maintainer_repo_feed_atom, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    MaintainerRepoFeedAtom,
    #[get("/maintainer/{maintainer_name}/problems-for-repo/{repository_name}", handler = handlers::maintainer_problems, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    MaintainerProblems,

    // Repositories
    #[get("/repositories/statistics", handler = handlers::repositories_statistics_default, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesStatistics,
    #[get("/repositories/statistics/{sorting}", handler = handlers::repositories_statistics_sorted, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesStatisticsSorted,
    #[get("/repositories/packages", handler = handlers::repositories_packages, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesPackages,
    #[get("/repositories/graphs", handler = handlers::repositories_graphs, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesGraphs,
    #[get("/repositories/updates", handler = handlers::repositories_updates, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesUpdates,
    #[get("/repositories/fields", handler = handlers::repositories_fields, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesFields,

    // Repository
    #[get("/repository/{repository_name}", handler = handlers::repository, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    Repository,
    #[get("/repository/{repository_name}/feed", handler = handlers::repository_feed, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoryFeed,
    #[get("/repository/{repository_name}/feed/atom", handler = handlers::repository_feed_atom, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoryFeedAtom,
    #[get("/repository/{repository_name}/problems", handler = handlers::repository_problems, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoryProblems,

    // Tools
    #[get("/tools", handler = handlers::tools, props = RouteProps { section: Section::Tools, ..Default::default() })]
    Tools,
    #[get("/tools/project-by", handler = handlers::project_by, props = RouteProps { section: Section::Tools, ..Default::default() })]
    ToolProjectBy,
    #[get("/tools/trending", handler = handlers::trending, props = RouteProps { section: Section::Tools, ..Default::default() })]
    Trending,
    #[get("/tools/important-updates", handler = handlers::important_updates, props = RouteProps { section: Section::Tools, ..Default::default() })]
    ImportantUpdates,

    // Security
    #[get("/security/recent-cves", handler = handlers::recent_cves, props = RouteProps { section: Section::Security, ..Default::default() })]
    SecurityRecentCves,
    #[get("/security/recent-cpes", handler = handlers::recent_cpes, props = RouteProps { section: Section::Security, ..Default::default() })]
    SecurityRecentCpes,

    // News/Docs
    #[get("/news", handler = handlers::news, props = RouteProps { section: Section::News, ..Default::default() })]
    News,
    #[get("/docs", handler = handlers::docs, props = RouteProps { section: Section::Docs, ..Default::default() })]
    Docs,
    #[get("/docs/about", handler = handlers::docs_about, props = RouteProps { section: Section::Docs, ..Default::default() })]
    DocsAbout,
    #[get("/docs/bots", handler = handlers::docs_bots, props = RouteProps { section: Section::Docs, ..Default::default() })]
    DocsBots,
    #[get("/docs/not_supported", handler = handlers::docs_not_supported, props = RouteProps { section: Section::Docs, ..Default::default() })]
    DocsNotSupported,
    #[get("/docs/requirements", handler = handlers::docs_requirements, props = RouteProps { section: Section::Docs, ..Default::default() })]
    DocsRequirements,
    #[get("/api", handler = handlers::api_v1, props = RouteProps { section: Section::Docs, ..Default::default() })]
    Api, // XXX: do we need this duplicate route
    #[get("/api/v1", handler = handlers::api_v1, props = RouteProps { section: Section::Docs, ..Default::default() })]
    ApiV1,

    // Misc
    #[get("/log/{run_id}", handler = handlers::log)]
    Log,
    #[get("/favicon.ico", handler = handlers::favicon)]
    Favicon,

    // Misc
    #[get("/link/{*url}", handler = handlers::link)]
    Link,

    // API
    #[get("/api/v1/projects/", handler = handlers::api_v1_projects)]
    ApiV1Projects,
    #[get("/api/v1/projects/{bound}/", handler = handlers::api_v1_projects_bounded)]
    ApiV1ProjectsBounded,
    #[get("/api/v1/project/{project_name}", handler = handlers::api_v1_project)]
    ApiV1Project,
    #[get("/api/v1/repository/{repository_name}/problems", handler = handlers::api_v1_repository_problems)]
    ApiV1RepositoryProblems,
    #[get("/api/v1/maintainer/{maintainer_name}/problems-for-repo/{repository_name}", handler = handlers::api_v1_maintainer_problems)]
    ApiV1MaintainerProblems,

    // Graph
    #[get("/graph/total/packages.svg", handler = handlers::graph_total_packages)]
    GraphTotalPackages,
    #[get("/graph/total/projects.svg", handler = handlers::graph_total_projects)]
    GraphTotalProjects,
    #[get("/graph/total/maintainers.svg", handler = handlers::graph_total_maintainers)]
    GraphTotalMaintainers,
    #[get("/graph/total/problems.svg", handler = handlers::graph_total_problems)]
    GraphTotalProblems,

    #[get("/graph/repo/{repository_name}/problems.svg", handler = handlers::graph_repository_problems)]
    GraphRepoProblems,
    #[get("/graph/repo/{repository_name}/maintainers.svg", handler = handlers::graph_repository_maintainers)]
    GraphRepoMaintainers,
    #[get("/graph/repo/{repository_name}/projects_total.svg", handler = handlers::graph_repository_projects_total)]
    GraphRepoProjectsTotal,
    #[get("/graph/repo/{repository_name}/projects_unique.svg", handler = handlers::graph_repository_projects_unique)]
    GraphRepoProjectsUnique,
    #[get("/graph/repo/{repository_name}/projects_newest.svg", handler = handlers::graph_repository_projects_newest)]
    GraphRepoProjectsNewest,
    #[get("/graph/repo/{repository_name}/projects_outdated.svg", handler = handlers::graph_repository_projects_outdated)]
    GraphRepoProjectsOutdated,
    #[get("/graph/repo/{repository_name}/projects_problematic.svg", handler = handlers::graph_repository_projects_problematic)]
    GraphRepoProjectsProblematic,
    #[get("/graph/repo/{repository_name}/projects_vulnerable.svg", handler = handlers::graph_repository_projects_vulnerable)]
    GraphRepoProjectsVulnerable,

    #[get("/graph/repo/{repository_name}/projects_newest_percent.svg", handler = handlers::graph_repository_projects_newest_percent)]
    GraphRepoProjectsNewestPercent,
    #[get("/graph/repo/{repository_name}/projects_outdated_percent.svg", handler = handlers::graph_repository_projects_outdated_percent)]
    GraphRepoProjectsOutdatedPercent,
    #[get("/graph/repo/{repository_name}/projects_unique_percent.svg", handler = handlers::graph_repository_projects_unique_percent)]
    GraphRepoProjectsUniquePercent,
    #[get("/graph/repo/{repository_name}/projects_problematic_percent.svg", handler = handlers::graph_repository_projects_problematic_percent)]
    GraphRepoProjectsProblematicPercent,
    #[get("/graph/repo/{repository_name}/projects_vulnerable_percent.svg", handler = handlers::graph_repository_projects_vulnerable_percent)]
    GraphRepoProjectsVulnerablePercent,

    #[get("/graph/repo/{repository_name}/problems_per_1000_projects.svg", handler = handlers::graph_repository_problems_per_1000_projects)]
    GraphRepoProblemsPer1000Projects,
    #[get("/graph/repo/{repository_name}/projects_per_maintainer.svg", handler = handlers::graph_repository_projects_per_maintainer)]
    GraphRepoProjectsPerMaintainer,

    #[get("/graph/map_repo_size_fresh.svg", handler = handlers::graph_map_repo_size_fresh)]
    GraphMapRepoSizeFresh,

    // Opensearch
    #[get("/opensearch/project.xml", handler = handlers::opensearch_project)]
    OpensearchProject,
    #[get("/opensearch/maintainer.xml", handler = handlers::opensearch_maintainer)]
    OpensearchMaintainer,

    // Badges
    #[get("/badge/tiny-repos/{project_name}.svg", handler = handlers::badge_tiny_repos, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeTinyRepos,
    #[get("/badge/version-for-repo/{repository_name}/{project_name}.svg", handler = handlers::badge_version_for_repo, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeVersionForRepo,
    #[get("/badge/vertical-allrepos/{project_name}.svg", handler = handlers::badge_vertical_allrepos, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeVerticalAllRepos,
    #[get("/badge/latest-versions/{project_name}.svg", handler = handlers::badge_latest_versions, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeLatestVersions,
    #[get("/badge/repository-big/{repository_name}.svg", handler = handlers::badge_repository_big, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeRepositoryBig,
    #[get("/badge/versions-matrix.svg", handler = handlers::badge_versions_matrix, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeVersionsMatrix,

    // Legacy redirects
    #[get("/badge/version-only-for-repo/{repository_name}/{project_name}.svg", handler = handlers::legacy_badge_version_only_for_repo, props = RouteProps { allow_embedding: true, ..Default::default() })]
    LegacyBadgeVersionOnlyForRepo,
    #[get("/project/{project_name}", handler = handlers::legacy_metapackage_versions)]
    LegacyProject,
    #[get("/metapackage/{project_name}", handler = handlers::legacy_metapackage_versions)]
    LegacyMetapackage,
    #[get("/metapackage/{project_name}/versions", handler = handlers::legacy_metapackage_versions)]
    LegacyMetapackageVersions,
    #[get("/metapackage/{project_name}/packages", handler = handlers::legacy_metapackage_packages)]
    LegacyMetapackagePackages,

    // Sitemaps
    #[get("/sitemaps/index.xml", handler = handlers::sitemap_index)]
    SitemapIndex,
    #[get("/sitemaps/main.xml", handler = handlers::sitemap_main)]
    SitemapMain,
    #[get("/sitemaps/repositories.xml", handler = handlers::sitemap_repositories)]
    SitemapRepositories,
    #[get("/sitemaps/maintainers.xml", handler = handlers::sitemap_maintainers)]
    SitemapMaintainers,
    #[get("/sitemaps/projects.xml", handler = handlers::sitemap_projects)]
    SitemapProjects,
}
