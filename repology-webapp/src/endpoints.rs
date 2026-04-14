// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum_myroutes::routes;

use crate::state::AppState;
use crate::views;

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
    #[allow(unused)] // will be used in heades middleware
    pub allow_embedding: bool,
}

// endpoint ordering:
// static -> index -> pages according to navbar -> supplementary pages -> supplementary endpoints
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[routes(props_type = RouteProps, state_type = Arc<AppState>)]
pub enum Endpoint {
    // Static
    #[get("/static/{file_name}", handler = views::static_file)]
    StaticFile,

    // Index
    #[get("/", handler = views::index)]
    Index,

    // Projects
    #[get("/projects/", handler = views::projects, props = RouteProps { section: Section::Projects, ..Default::default() })]
    Projects,
    #[get("/projects/{bound}/", handler = views::projects_bounded, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectsBounded,

    // Project
    #[get("/project/{project_name}/versions", handler = views::project_versions, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectVersions,
    #[get("/project/{project_name}/versions-compact", handler = views::project_versions_compact, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectVersionsCompact,
    #[get("/project/{project_name}/packages", handler = views::project_packages, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectPackages,
    #[get("/project/{project_name}/information", handler = views::project_information, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectInformation,
    #[get("/project/{project_name}/history", handler = views::project_history, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectHistory,
    #[get("/project/{project_name}/related", handler = views::project_related, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectRelated,
    #[get("/project/{project_name}/badges", handler = views::project_badges, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectBadges,
    #[get("/project/{project_name}/report", handler = views::project_report_get, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectReport,
    #[post("/project/{project_name}/report", handler = views::project_report_post, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectReportPost,
    #[get("/project/{project_name}/cves", handler = views::project_cves, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectCves,

    // Maintainers
    #[get("/maintainers/", handler = views::maintainers, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    Maintainers,
    #[get("/maintainers/{bound}/", handler = views::maintainers_bounded, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    MaintainersBounded,

    // Maintainer
    #[get("/maintainer/{maintainer_name}", handler = views::maintainer, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    Maintainer,
    #[get("/maintainer/{maintainer_name}/feed-for-repo/{repository_name}", handler = views::maintainer_repo_feed, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    MaintainerRepoFeed,
    #[get("/maintainer/{maintainer_name}/feed-for-repo/{repository_name}/atom", handler = views::maintainer_repo_feed_atom, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    MaintainerRepoFeedAtom,
    #[get("/maintainer/{maintainer_name}/problems-for-repo/{repository_name}", handler = views::maintainer_problems, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    MaintainerProblems,

    // Repositories
    #[get("/repositories/statistics", handler = views::repositories_statistics_default, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesStatistics,
    #[get("/repositories/statistics/{sorting}", handler = views::repositories_statistics_sorted, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesStatisticsSorted,
    #[get("/repositories/packages", handler = views::repositories_packages, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesPackages,
    #[get("/repositories/graphs", handler = views::repositories_graphs, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesGraphs,
    #[get("/repositories/updates", handler = views::repositories_updates, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesUpdates,
    #[get("/repositories/fields", handler = views::repositories_fields, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesFields,

    // Repository
    #[get("/repository/{repository_name}", handler = views::repository, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    Repository,
    #[get("/repository/{repository_name}/feed", handler = views::repository_feed, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoryFeed,
    #[get("/repository/{repository_name}/feed/atom", handler = views::repository_feed_atom, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoryFeedAtom,
    #[get("/repository/{repository_name}/problems", handler = views::repository_problems, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoryProblems,

    // Tools
    #[get("/tools", handler = views::tools, props = RouteProps { section: Section::Tools, ..Default::default() })]
    Tools,
    #[get("/tools/project-by", handler = views::project_by, props = RouteProps { section: Section::Tools, ..Default::default() })]
    ToolProjectBy,
    #[get("/tools/trending", handler = views::trending, props = RouteProps { section: Section::Tools, ..Default::default() })]
    Trending,
    #[get("/tools/important-updates", handler = views::important_updates, props = RouteProps { section: Section::Tools, ..Default::default() })]
    ImportantUpdates,

    // Security
    #[get("/security/recent-cves", handler = views::recent_cves, props = RouteProps { section: Section::Security, ..Default::default() })]
    SecurityRecentCves,
    #[get("/security/recent-cpes", handler = views::recent_cpes, props = RouteProps { section: Section::Security, ..Default::default() })]
    SecurityRecentCpes,

    // News/Docs
    #[get("/news", handler = views::news, props = RouteProps { section: Section::News, ..Default::default() })]
    News,
    #[get("/docs", handler = views::docs, props = RouteProps { section: Section::Docs, ..Default::default() })]
    Docs,
    #[get("/docs/about", handler = views::docs_about, props = RouteProps { section: Section::Docs, ..Default::default() })]
    DocsAbout,
    #[get("/docs/bots", handler = views::docs_bots, props = RouteProps { section: Section::Docs, ..Default::default() })]
    DocsBots,
    #[get("/docs/not_supported", handler = views::docs_not_supported, props = RouteProps { section: Section::Docs, ..Default::default() })]
    DocsNotSupported,
    #[get("/docs/requirements", handler = views::docs_requirements, props = RouteProps { section: Section::Docs, ..Default::default() })]
    DocsRequirements,
    #[get("/api", handler = views::api_v1, props = RouteProps { section: Section::Docs, ..Default::default() })]
    Api, // XXX: do we need this duplicate endpoints
    #[get("/api/v1", handler = views::api_v1, props = RouteProps { section: Section::Docs, ..Default::default() })]
    ApiV1,

    // Misc
    #[get("/log/{run_id}", handler = views::log)]
    Log,
    #[get("/favicon.ico", handler = views::favicon)]
    Favicon,

    // Misc
    #[get("/link/{*url}", handler = views::link)]
    Link,

    // API
    #[get("/api/v1/projects/", handler = views::api_v1_projects)]
    ApiV1Projects,
    #[get("/api/v1/projects/{bound}/", handler = views::api_v1_projects_bounded)]
    ApiV1ProjectsBounded,
    #[get("/api/v1/project/{project_name}", handler = views::api_v1_project)]
    ApiV1Project,
    #[get("/api/v1/repository/{repository_name}/problems", handler = views::api_v1_repository_problems)]
    ApiV1RepositoryProblems,
    #[get("/api/v1/maintainer/{maintainer_name}/problems-for-repo/{repository_name}", handler = views::api_v1_maintainer_problems)]
    ApiV1MaintainerProblems,

    // Graph
    #[get("/graph/total/packages.svg", handler = views::graph_total_packages)]
    GraphTotalPackages,
    #[get("/graph/total/projects.svg", handler = views::graph_total_projects)]
    GraphTotalProjects,
    #[get("/graph/total/maintainers.svg", handler = views::graph_total_maintainers)]
    GraphTotalMaintainers,
    #[get("/graph/total/problems.svg", handler = views::graph_total_problems)]
    GraphTotalProblems,

    #[get("/graph/repo/{repository_name}/problems.svg", handler = views::graph_repository_problems)]
    GraphRepoProblems,
    #[get("/graph/repo/{repository_name}/maintainers.svg", handler = views::graph_repository_maintainers)]
    GraphRepoMaintainers,
    #[get("/graph/repo/{repository_name}/projects_total.svg", handler = views::graph_repository_projects_total)]
    GraphRepoProjectsTotal,
    #[get("/graph/repo/{repository_name}/projects_unique.svg", handler = views::graph_repository_projects_unique)]
    GraphRepoProjectsUnique,
    #[get("/graph/repo/{repository_name}/projects_newest.svg", handler = views::graph_repository_projects_newest)]
    GraphRepoProjectsNewest,
    #[get("/graph/repo/{repository_name}/projects_outdated.svg", handler = views::graph_repository_projects_outdated)]
    GraphRepoProjectsOutdated,
    #[get("/graph/repo/{repository_name}/projects_problematic.svg", handler = views::graph_repository_projects_problematic)]
    GraphRepoProjectsProblematic,
    #[get("/graph/repo/{repository_name}/projects_vulnerable.svg", handler = views::graph_repository_projects_vulnerable)]
    GraphRepoProjectsVulnerable,

    #[get("/graph/repo/{repository_name}/projects_newest_percent.svg", handler = views::graph_repository_projects_newest_percent)]
    GraphRepoProjectsNewestPercent,
    #[get("/graph/repo/{repository_name}/projects_outdated_percent.svg", handler = views::graph_repository_projects_outdated_percent)]
    GraphRepoProjectsOutdatedPercent,
    #[get("/graph/repo/{repository_name}/projects_unique_percent.svg", handler = views::graph_repository_projects_unique_percent)]
    GraphRepoProjectsUniquePercent,
    #[get("/graph/repo/{repository_name}/projects_problematic_percent.svg", handler = views::graph_repository_projects_problematic_percent)]
    GraphRepoProjectsProblematicPercent,
    #[get("/graph/repo/{repository_name}/projects_vulnerable_percent.svg", handler = views::graph_repository_projects_vulnerable_percent)]
    GraphRepoProjectsVulnerablePercent,

    #[get("/graph/repo/{repository_name}/problems_per_1000_projects.svg", handler = views::graph_repository_problems_per_1000_projects)]
    GraphRepoProblemsPer1000Projects,
    #[get("/graph/repo/{repository_name}/projects_per_maintainer.svg", handler = views::graph_repository_projects_per_maintainer)]
    GraphRepoProjectsPerMaintainer,

    #[get("/graph/map_repo_size_fresh.svg", handler = views::graph_map_repo_size_fresh)]
    GraphMapRepoSizeFresh,

    // Opensearch
    #[get("/opensearch/project.xml", handler = views::opensearch_project)]
    OpensearchProject,
    #[get("/opensearch/maintainer.xml", handler = views::opensearch_maintainer)]
    OpensearchMaintainer,

    // Badges
    #[get("/badge/tiny-repos/{project_name}.svg", handler = views::badge_tiny_repos, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeTinyRepos,
    #[get("/badge/version-for-repo/{repository_name}/{project_name}.svg", handler = views::badge_version_for_repo, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeVersionForRepo,
    #[get("/badge/vertical-allrepos/{project_name}.svg", handler = views::badge_vertical_allrepos, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeVerticalAllRepos,
    #[get("/badge/latest-versions/{project_name}.svg", handler = views::badge_latest_versions, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeLatestVersions,
    #[get("/badge/repository-big/{repository_name}.svg", handler = views::badge_repository_big, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeRepositoryBig,
    #[get("/badge/versions-matrix.svg", handler = views::badge_versions_matrix, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeVersionsMatrix,

    // Legacy redirects
    #[get("/badge/version-only-for-repo/{repository_name}/{project_name}.svg", handler = views::legacy_badge_version_only_for_repo, props = RouteProps { allow_embedding: true, ..Default::default() })]
    LegacyBadgeVersionOnlyForRepo,
    #[get("/project/{project_name}", handler = views::legacy_metapackage_versions)]
    LegacyProject,
    #[get("/metapackage/{project_name}", handler = views::legacy_metapackage_versions)]
    LegacyMetapackage,
    #[get("/metapackage/{project_name}/versions", handler = views::legacy_metapackage_versions)]
    LegacyMetapackageVersions,
    #[get("/metapackage/{project_name}/packages", handler = views::legacy_metapackage_packages)]
    LegacyMetapackagePackages,

    // Sitemaps
    #[get("/sitemaps/index.xml", handler = views::sitemap_index)]
    SitemapIndex,
    #[get("/sitemaps/main.xml", handler = views::sitemap_main)]
    SitemapMain,
    #[get("/sitemaps/repositories.xml", handler = views::sitemap_repositories)]
    SitemapRepositories,
    #[get("/sitemaps/maintainers.xml", handler = views::sitemap_maintainers)]
    SitemapMaintainers,
    #[get("/sitemaps/projects.xml", handler = views::sitemap_projects)]
    SitemapProjects,
}
