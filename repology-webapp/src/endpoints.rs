// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum_myroutes::routes;
use strum::EnumProperty;
use strum_macros::{EnumString, IntoStaticStr};

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
    pub allow_embedding: bool,
}

// endpoint ordering:
// static -> index -> pages according to navbar -> supplementary pages -> supplementary endpoints
#[derive(EnumProperty, IntoStaticStr, EnumString, Clone, Copy, Debug, PartialEq, Eq)]
#[routes(props_type = RouteProps, state_type = Arc<AppState>)]
pub enum Endpoint {
    // Static
    #[strum(props(path = "/static/{file_name}"))]
    #[get("/static/{file_name}", handler = views::static_file)]
    StaticFile,

    // Index
    #[strum(props(path = "/"))]
    #[get("/", handler = views::index)]
    Index,

    // Projects
    #[strum(props(path = "/projects/", section = "Projects"))]
    #[get("/projects/", handler = views::projects, props = RouteProps { section: Section::Projects, ..Default::default() })]
    Projects,
    #[strum(props(path = "/projects/{bound}/", section = "Projects"))]
    #[get("/projects/{bound}/", handler = views::projects_bounded, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectsBounded,

    // Project
    #[strum(props(path = "/project/{project_name}/versions", section = "Projects"))]
    #[get("/project/{project_name}/versions", handler = views::project_versions, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectVersions,
    #[strum(props(
        path = "/project/{project_name}/versions-compact",
        section = "Projects"
    ))]
    #[get("/project/{project_name}/versions-compact", handler = views::project_versions_compact, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectVersionsCompact,
    #[strum(props(path = "/project/{project_name}/packages", section = "Projects"))]
    #[get("/project/{project_name}/packages", handler = views::project_packages, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectPackages,
    #[strum(props(path = "/project/{project_name}/information", section = "Projects"))]
    #[get("/project/{project_name}/information", handler = views::project_information, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectInformation,
    #[strum(props(path = "/project/{project_name}/history", section = "Projects"))]
    #[get("/project/{project_name}/history", handler = views::project_history, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectHistory,
    #[strum(props(path = "/project/{project_name}/related", section = "Projects"))]
    #[get("/project/{project_name}/related", handler = views::project_related, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectRelated,
    #[strum(props(path = "/project/{project_name}/badges", section = "Projects"))]
    #[get("/project/{project_name}/badges", handler = views::project_badges, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectBadges,
    #[strum(props(path = "/project/{project_name}/report", section = "Projects"))]
    #[get("/project/{project_name}/report", handler = views::project_report_get, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectReport,
    #[post("/project/{project_name}/report", handler = views::project_report_post, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectReportPost,
    #[strum(props(path = "/project/{project_name}/cves", section = "Projects"))]
    #[get("/project/{project_name}/cves", handler = views::project_cves, props = RouteProps { section: Section::Projects, ..Default::default() })]
    ProjectCves,

    // Maintainers
    #[strum(props(path = "/maintainers/", section = "Maintainers"))]
    #[get("/maintainers/", handler = views::maintainers, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    Maintainers,
    #[strum(props(path = "/maintainers/{bound}/", section = "Maintainers"))]
    #[get("/maintainers/{bound}/", handler = views::maintainers_bounded, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    MaintainersBounded,

    // Maintainer
    #[strum(props(path = "/maintainer/{maintainer_name}", section = "Maintainers"))]
    #[get("/maintainer/{maintainer_name}", handler = views::maintainer, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    Maintainer,
    #[strum(props(
        path = "/maintainer/{maintainer_name}/feed-for-repo/{repository_name}",
        section = "Maintainers"
    ))]
    #[get("/maintainer/{maintainer_name}/feed-for-repo/{repository_name}", handler = views::maintainer_repo_feed, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    MaintainerRepoFeed,
    #[strum(props(
        path = "/maintainer/{maintainer_name}/feed-for-repo/{repository_name}/atom",
        section = "Maintainers"
    ))]
    #[get("/maintainer/{maintainer_name}/feed-for-repo/{repository_name}/atom", handler = views::maintainer_repo_feed_atom, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    MaintainerRepoFeedAtom,
    #[strum(props(
        path = "/maintainer/{maintainer_name}/problems-for-repo/{repository_name}",
        section = "Maintainers"
    ))]
    #[get("/maintainer/{maintainer_name}/problems-for-repo/{repository_name}", handler = views::maintainer_problems, props = RouteProps { section: Section::Maintainers, ..Default::default() })]
    MaintainerProblems,

    // Repositories
    #[strum(props(path = "/repositories/statistics", section = "Repositories"))]
    #[get("/repositories/statistics", handler = views::repositories_statistics_default, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesStatistics,
    #[strum(props(path = "/repositories/statistics/{sorting}", section = "Repositories"))]
    #[get("/repositories/statistics/{sorting}", handler = views::repositories_statistics_sorted, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesStatisticsSorted,
    #[strum(props(path = "/repositories/packages", section = "Repositories"))]
    #[get("/repositories/packages", handler = views::repositories_packages, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesPackages,
    #[strum(props(path = "/repositories/graphs", section = "Repositories"))]
    #[get("/repositories/graphs", handler = views::repositories_graphs, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesGraphs,
    #[strum(props(path = "/repositories/updates", section = "Repositories"))]
    #[get("/repositories/updates", handler = views::repositories_updates, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesUpdates,
    #[strum(props(path = "/repositories/fields", section = "Repositories"))]
    #[get("/repositories/fields", handler = views::repositories_fields, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoriesFields,

    // Repository
    #[strum(props(path = "/repository/{repository_name}", section = "Repositories"))]
    #[get("/repository/{repository_name}", handler = views::repository, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    Repository,
    #[strum(props(path = "/repository/{repository_name}/feed", section = "Repositories"))]
    #[get("/repository/{repository_name}/feed", handler = views::repository_feed, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoryFeed,
    #[strum(props(
        path = "/repository/{repository_name}/feed/atom",
        section = "Repositories"
    ))]
    #[get("/repository/{repository_name}/feed/atom", handler = views::repository_feed_atom, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoryFeedAtom,
    #[strum(props(
        path = "/repository/{repository_name}/problems",
        section = "Repositories"
    ))]
    #[get("/repository/{repository_name}/problems", handler = views::repository_problems, props = RouteProps { section: Section::Repositories, ..Default::default() })]
    RepositoryProblems,

    // Tools
    #[strum(props(path = "/tools", section = "Tools"))]
    #[get("/tools", handler = views::tools, props = RouteProps { section: Section::Tools, ..Default::default() })]
    Tools,
    #[strum(props(path = "/tools/project-by", section = "Tools"))]
    #[get("/tools/project-by", handler = views::project_by, props = RouteProps { section: Section::Tools, ..Default::default() })]
    ToolProjectBy,
    #[strum(props(path = "/tools/trending", section = "Tools"))]
    #[get("/tools/trending", handler = views::trending, props = RouteProps { section: Section::Tools, ..Default::default() })]
    Trending,
    #[strum(props(path = "/tools/important-updates", section = "Tools"))]
    #[get("/tools/important-updates", handler = views::important_updates, props = RouteProps { section: Section::Tools, ..Default::default() })]
    ImportantUpdates,

    // Security
    #[strum(props(path = "/security/recent-cves", section = "Security"))]
    #[get("/security/recent-cves", handler = views::recent_cves, props = RouteProps { section: Section::Security, ..Default::default() })]
    SecurityRecentCves,
    #[strum(props(path = "/security/recent-cpes", section = "Security"))]
    #[get("/security/recent-cpes", handler = views::recent_cpes, props = RouteProps { section: Section::Security, ..Default::default() })]
    SecurityRecentCpes,

    // News/Docs
    #[strum(props(path = "/news", section = "News"))]
    #[get("/news", handler = views::news, props = RouteProps { section: Section::News, ..Default::default() })]
    News,
    #[strum(props(path = "/docs", section = "Docs"))]
    #[get("/docs", handler = views::docs, props = RouteProps { section: Section::Docs, ..Default::default() })]
    Docs,
    #[strum(props(path = "/docs/about", section = "Docs"))]
    #[get("/docs/about", handler = views::docs_about, props = RouteProps { section: Section::Docs, ..Default::default() })]
    DocsAbout,
    #[strum(props(path = "/docs/bots", section = "Docs"))]
    #[get("/docs/bots", handler = views::docs_bots, props = RouteProps { section: Section::Docs, ..Default::default() })]
    DocsBots,
    #[strum(props(path = "/docs/not_supported", section = "Docs"))]
    #[get("/docs/not_supported", handler = views::docs_not_supported, props = RouteProps { section: Section::Docs, ..Default::default() })]
    DocsNotSupported,
    #[strum(props(path = "/docs/requirements", section = "Docs"))]
    #[get("/docs/requirements", handler = views::docs_requirements, props = RouteProps { section: Section::Docs, ..Default::default() })]
    DocsRequirements,
    #[strum(props(path = "/api", section = "Docs"))]
    #[get("/api", handler = views::api_v1, props = RouteProps { section: Section::Docs, ..Default::default() })]
    Api, // XXX: do we need this duplicate endpoints
    #[strum(props(path = "/api/v1", section = "Docs"))]
    #[get("/api/v1", handler = views::api_v1, props = RouteProps { section: Section::Docs, ..Default::default() })]
    ApiV1,

    // Misc
    #[strum(props(path = "/log/{run_id}"))]
    #[get("/log/{run_id}", handler = views::log)]
    Log,
    #[strum(props(path = "/favicon.ico"))]
    #[get("/favicon.ico", handler = views::favicon)]
    Favicon,

    // Misc
    #[strum(props(path = "/link/{*url}"))]
    #[get("/link/{*url}", handler = views::link)]
    Link,

    // API
    #[strum(props(path = "/api/v1/projects/"))]
    #[get("/api/v1/projects/", handler = views::api_v1_projects)]
    ApiV1Projects,
    #[strum(props(path = "/api/v1/projects/{bound}/"))]
    #[get("/api/v1/projects/{bound}/", handler = views::api_v1_projects_bounded)]
    ApiV1ProjectsBounded,
    #[strum(props(path = "/api/v1/project/{project_name}"))]
    #[get("/api/v1/project/{project_name}", handler = views::api_v1_project)]
    ApiV1Project,
    #[strum(props(path = "/api/v1/repository/{repository_name}/problems"))]
    #[get("/api/v1/repository/{repository_name}/problems", handler = views::api_v1_repository_problems)]
    ApiV1RepositoryProblems,
    #[strum(props(
        path = "/api/v1/maintainer/{maintainer_name}/problems-for-repo/{repository_name}"
    ))]
    #[get("/api/v1/maintainer/{maintainer_name}/problems-for-repo/{repository_name}", handler = views::api_v1_maintainer_problems)]
    ApiV1MaintainerProblems,

    // Graph
    #[strum(props(path = "/graph/total/packages.svg"))]
    #[get("/graph/total/packages.svg", handler = views::graph_total_packages)]
    GraphTotalPackages,
    #[strum(props(path = "/graph/total/projects.svg"))]
    #[get("/graph/total/projects.svg", handler = views::graph_total_projects)]
    GraphTotalProjects,
    #[strum(props(path = "/graph/total/maintainers.svg"))]
    #[get("/graph/total/maintainers.svg", handler = views::graph_total_maintainers)]
    GraphTotalMaintainers,
    #[strum(props(path = "/graph/total/problems.svg"))]
    #[get("/graph/total/problems.svg", handler = views::graph_total_problems)]
    GraphTotalProblems,

    #[strum(props(path = "/graph/repo/{repository_name}/problems.svg"))]
    #[get("/graph/repo/{repository_name}/problems.svg", handler = views::graph_repository_problems)]
    GraphRepoProblems,
    #[strum(props(path = "/graph/repo/{repository_name}/maintainers.svg"))]
    #[get("/graph/repo/{repository_name}/maintainers.svg", handler = views::graph_repository_maintainers)]
    GraphRepoMaintainers,
    #[strum(props(path = "/graph/repo/{repository_name}/projects_total.svg"))]
    #[get("/graph/repo/{repository_name}/projects_total.svg", handler = views::graph_repository_projects_total)]
    GraphRepoProjectsTotal,
    #[strum(props(path = "/graph/repo/{repository_name}/projects_unique.svg"))]
    #[get("/graph/repo/{repository_name}/projects_unique.svg", handler = views::graph_repository_projects_unique)]
    GraphRepoProjectsUnique,
    #[strum(props(path = "/graph/repo/{repository_name}/projects_newest.svg"))]
    #[get("/graph/repo/{repository_name}/projects_newest.svg", handler = views::graph_repository_projects_newest)]
    GraphRepoProjectsNewest,
    #[strum(props(path = "/graph/repo/{repository_name}/projects_outdated.svg"))]
    #[get("/graph/repo/{repository_name}/projects_outdated.svg", handler = views::graph_repository_projects_outdated)]
    GraphRepoProjectsOutdated,
    #[strum(props(path = "/graph/repo/{repository_name}/projects_problematic.svg"))]
    #[get("/graph/repo/{repository_name}/projects_problematic.svg", handler = views::graph_repository_projects_problematic)]
    GraphRepoProjectsProblematic,
    #[strum(props(path = "/graph/repo/{repository_name}/projects_vulnerable.svg"))]
    #[get("/graph/repo/{repository_name}/projects_vulnerable.svg", handler = views::graph_repository_projects_vulnerable)]
    GraphRepoProjectsVulnerable,

    #[strum(props(path = "/graph/repo/{repository_name}/projects_newest_percent.svg"))]
    #[get("/graph/repo/{repository_name}/projects_newest_percent.svg", handler = views::graph_repository_projects_newest_percent)]
    GraphRepoProjectsNewestPercent,
    #[strum(props(path = "/graph/repo/{repository_name}/projects_outdated_percent.svg"))]
    #[get("/graph/repo/{repository_name}/projects_outdated_percent.svg", handler = views::graph_repository_projects_outdated_percent)]
    GraphRepoProjectsOutdatedPercent,
    #[strum(props(path = "/graph/repo/{repository_name}/projects_unique_percent.svg"))]
    #[get("/graph/repo/{repository_name}/projects_unique_percent.svg", handler = views::graph_repository_projects_unique_percent)]
    GraphRepoProjectsUniquePercent,
    #[strum(props(path = "/graph/repo/{repository_name}/projects_problematic_percent.svg"))]
    #[get("/graph/repo/{repository_name}/projects_problematic_percent.svg", handler = views::graph_repository_projects_problematic_percent)]
    GraphRepoProjectsProblematicPercent,
    #[strum(props(path = "/graph/repo/{repository_name}/projects_vulnerable_percent.svg"))]
    #[get("/graph/repo/{repository_name}/projects_vulnerable_percent.svg", handler = views::graph_repository_projects_vulnerable_percent)]
    GraphRepoProjectsVulnerablePercent,

    #[strum(props(path = "/graph/repo/{repository_name}/problems_per_1000_projects.svg"))]
    #[get("/graph/repo/{repository_name}/problems_per_1000_projects.svg", handler = views::graph_repository_problems_per_1000_projects)]
    GraphRepoProblemsPer1000Projects,
    #[strum(props(path = "/graph/repo/{repository_name}/projects_per_maintainer.svg"))]
    #[get("/graph/repo/{repository_name}/projects_per_maintainer.svg", handler = views::graph_repository_projects_per_maintainer)]
    GraphRepoProjectsPerMaintainer,

    #[strum(props(path = "/graph/map_repo_size_fresh.svg"))]
    #[get("/graph/map_repo_size_fresh.svg", handler = views::graph_map_repo_size_fresh)]
    GraphMapRepoSizeFresh,

    // Opensearch
    #[strum(props(path = "/opensearch/project.xml"))]
    #[get("/opensearch/project.xml", handler = views::opensearch_project)]
    OpensearchProject,
    #[strum(props(path = "/opensearch/maintainer.xml"))]
    #[get("/opensearch/maintainer.xml", handler = views::opensearch_maintainer)]
    OpensearchMaintainer,

    // Badges
    #[strum(props(path = "/badge/tiny-repos/{project_name}.svg"))]
    #[get("/badge/tiny-repos/{project_name}.svg", handler = views::badge_tiny_repos, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeTinyRepos,
    #[strum(props(path = "/badge/version-for-repo/{repository_name}/{project_name}.svg"))]
    #[get("/badge/version-for-repo/{repository_name}/{project_name}.svg", handler = views::badge_version_for_repo, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeVersionForRepo,
    #[strum(props(path = "/badge/vertical-allrepos/{project_name}.svg"))]
    #[get("/badge/vertical-allrepos/{project_name}.svg", handler = views::badge_vertical_allrepos, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeVerticalAllRepos,
    #[strum(props(path = "/badge/latest-versions/{project_name}.svg"))]
    #[get("/badge/latest-versions/{project_name}.svg", handler = views::badge_latest_versions, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeLatestVersions,
    #[strum(props(path = "/badge/repository-big/{repository_name}.svg"))]
    #[get("/badge/repository-big/{repository_name}.svg", handler = views::badge_repository_big, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeRepositoryBig,
    #[strum(props(path = "/badge/versions-matrix.svg"))]
    #[get("/badge/versions-matrix.svg", handler = views::badge_versions_matrix, props = RouteProps { allow_embedding: true, ..Default::default() })]
    BadgeVersionsMatrix,

    // Legacy redirects
    #[strum(props(path = "/badge/version-only-for-repo/{repository_name}/{project_name}.svg"))]
    #[get("/badge/version-only-for-repo/{repository_name}/{project_name}.svg", handler = views::legacy_badge_version_only_for_repo, props = RouteProps { allow_embedding: true, ..Default::default() })]
    LegacyBadgeVersionOnlyForRepo,
    #[strum(props(path = "/project/{project_name}"))]
    #[get("/project/{project_name}", handler = views::legacy_metapackage_versions)]
    LegacyProject,
    #[strum(props(path = "/metapackage/{project_name}"))]
    #[get("/metapackage/{project_name}", handler = views::legacy_metapackage_versions)]
    LegacyMetapackage,
    #[strum(props(path = "/metapackage/{project_name}/versions"))]
    #[get("/metapackage/{project_name}/versions", handler = views::legacy_metapackage_versions)]
    LegacyMetapackageVersions,
    #[strum(props(path = "/metapackage/{project_name}/packages"))]
    #[get("/metapackage/{project_name}/packages", handler = views::legacy_metapackage_packages)]
    LegacyMetapackagePackages,

    // Sitemaps
    #[strum(props(path = "/sitemaps/index.xml"))]
    #[get("/sitemaps/index.xml", handler = views::sitemap_index)]
    SitemapIndex,
    #[strum(props(path = "/sitemaps/main.xml"))]
    #[get("/sitemaps/main.xml", handler = views::sitemap_main)]
    SitemapMain,
    #[strum(props(path = "/sitemaps/repositories.xml"))]
    #[get("/sitemaps/repositories.xml", handler = views::sitemap_repositories)]
    SitemapRepositories,
    #[strum(props(path = "/sitemaps/maintainers.xml"))]
    #[get("/sitemaps/maintainers.xml", handler = views::sitemap_maintainers)]
    SitemapMaintainers,
    #[strum(props(path = "/sitemaps/projects.xml"))]
    #[get("/sitemaps/projects.xml", handler = views::sitemap_projects)]
    SitemapProjects,
}

impl Endpoint {
    pub fn is_section(&self, section: Section) -> bool {
        self.props().section == section
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(
            Endpoint::BadgeVersionForRepo.path(),
            "/badge/version-for-repo/{repository_name}/{project_name}.svg"
        );
    }

    #[test]
    fn test_name() {
        assert_eq!(Endpoint::BadgeVersionForRepo.name(), "BadgeVersionForRepo");
    }
}
