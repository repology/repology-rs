// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use anyhow::{anyhow, bail, Error};
use serde_json::Value;
use strum::EnumProperty;
use strum_macros::{EnumString, IntoStaticStr};

// endpoint ordering:
// static -> index -> pages according to navbar -> supplementary pages -> supplementary endpoints
#[derive(EnumProperty, IntoStaticStr, EnumString, Clone, Copy, Debug)]
pub enum Endpoint {
    // Static
    #[strum(props(path = "/static/:file_name"))]
    StaticFile,

    // Tools
    #[strum(props(path = "/tools", section = "Tools"))]
    Tools,

    // News/Docs
    #[strum(props(path = "/news", section = "News"))]
    News,
    #[strum(props(path = "/docs", section = "Docs"))]
    Docs,
    #[strum(props(path = "/docs/about", section = "Docs"))]
    DocsAbout,
    #[strum(props(path = "/docs/bots", section = "Docs"))]
    DocsBots,
    #[strum(props(path = "/docs/not_supported", section = "Docs"))]
    DocsNotSupported,
    #[strum(props(path = "/docs/requirements", section = "Docs"))]
    DocsRequirements,
    #[strum(props(path = "/api", section = "Docs"))]
    Api, // XXX: do we need this duplicate endpoints
    #[strum(props(path = "/api/v1", section = "Docs"))]
    ApiV1,

    // Misc
    #[strum(props(path = "/log/:run_id"))]
    Log,

    // API
    #[strum(props(path = "/api/v1/project/:project_name"))]
    ApiV1Project,

    // Badges
    #[strum(props(path = "/badge/tiny-repos/:project_name.svg"))]
    BadgeTinyRepos,
    #[strum(props(path = "/badge/version-for-repo/:repository_name/:project_name.svg"))]
    BadgeVersionForRepo,
    #[strum(props(path = "/badge/vertical-allrepos/:project_name.svg"))]
    BadgeVerticalAllRepos,
    #[strum(props(path = "/badge/latest-versions/:project_name.svg"))]
    BadgeLatestVersions,

    //
    // not implemented yet
    //

    // Index
    #[strum(props(path = "/"))]
    Index,

    // Projects
    #[strum(props(path = "/projects/", section = "Projects"))]
    Projects,
    #[strum(props(path = "/projects/:bound/", section = "Projects"))]
    ProjectsBounded,

    // Project
    #[strum(props(path = "/project/:project/versions", section = "Projects"))]
    ProjectVersions,
    #[strum(props(path = "/project/:name/versions-compact", section = "Projects"))]
    ProjectVersionsCompact,
    #[strum(props(path = "/project/:name/packages", section = "Projects"))]
    ProjectPackages,
    #[strum(props(path = "/project/:name/information", section = "Projects"))]
    ProjectInformation,
    #[strum(props(path = "/project/:name/history", section = "Projects"))]
    ProjectHistory,
    #[strum(props(path = "/project/:name/related", section = "Projects"))]
    ProjectRelated,
    #[strum(props(path = "/project/:name/badges", section = "Projects"))]
    ProjectBadges,
    #[strum(props(path = "/project/:name/report", section = "Projects"))] // GET + POST
    ProjectReport,
    #[strum(props(path = "/project/:name/cves", section = "Projects"))]
    ProjectCves,

    // Maintainers
    #[strum(props(path = "/maintainers/", section = "Maintainers"))]
    Maintainers,
    #[strum(props(path = "/maintainers/:bound/", section = "Maintainers"))]
    MaintainersBounted,

    // Maintainer
    #[strum(props(path = "/maintainer/:maintainer", section = "Maintainers"))]
    Maintainer,
    #[strum(props(
        path = "/maintainer/:maintainer/problems-for-repo/:repo",
        section = "Maintainers"
    ))]
    MaintainerProblems,
    #[strum(props(
        path = "/maintainer/:maintainer/feed-for-repo/:repo",
        section = "Maintainers"
    ))]
    MaintainerRepoFeed,
    #[strum(props(
        path = "/maintainer/:maintainer/feed-for-repo/:repo/atom",
        section = "Maintainers"
    ))]
    MaintainerRepoFeedAtom,

    // Repositories
    #[strum(props(path = "/repositories/statistics", section = "Repositories"))]
    RepositoriesStatistics,
    #[strum(props(path = "/repositories/statistics/:sorting", section = "Repositories"))]
    RepositoriesStatisticsSorted,
    #[strum(props(path = "/repositories/packages", section = "Repositories"))]
    RepositoriesPackages,
    #[strum(props(path = "/repositories/updates", section = "Repositories"))]
    RepositoriesUpdates,
    #[strum(props(path = "/repositories/graphs", section = "Repositories"))]
    RepositoriesGraphs,
    #[strum(props(path = "/repositories/fields", section = "Repositories"))]
    RepositoriesFields,

    // Repository
    #[strum(props(path = "/repository/:repo", section = "Repositories"))]
    Repository,
    #[strum(props(path = "/repository/:repo/problems", section = "Repositories"))]
    RepositoryProblems,
    #[strum(props(path = "/repository/:repo/feed", section = "Repositories"))]
    RepositoryFeed,
    #[strum(props(path = "/repository/:repo/feed/atom", section = "Repositories"))]
    RepositoryFeedAtom,

    // Tools
    #[strum(props(path = "/tools/project-by", section = "Tools"))]
    ToolProjectBy,
    #[strum(props(path = "/tools/trending", section = "Tools"))]
    Trending,
    #[strum(props(path = "/tools/important_updates", section = "Experimental"))]
    ImportantUpdates,

    // Security
    #[strum(props(path = "/security/recent-cves", section = "Security"))]
    SecurityRecentCves,
    #[strum(props(path = "/security/recent-cpes", section = "Security"))]
    SecurityRecentCpes,

    // Admin
    #[strum(props(path = "/admin", section = "Admin"))] // GET + POST
    Admin,
    #[strum(props(path = "/admin/reports/unprocessed/", section = "Admin"))] // GET + POST
    AdminReportsUnprocessed,
    #[strum(props(path = "/admin/reports/recent/", section = "Admin"))] // GET + POST
    AdminReportsRecent,
    #[strum(props(path = "/admin/updates", section = "Admin"))]
    AdminUpdates,
    #[strum(props(path = "/admin/redirects", section = "Admin"))] // GET + POST
    AdminRedirects,
    #[strum(props(path = "/admin/name_samples", section = "Admin"))]
    AdminNameSamples,
    #[strum(props(path = "/admin/cpes", section = "Admin"))] // GET + POST
    AdminCpes,
    #[strum(props(path = "/admin/cve_misses", section = "Admin"))] // GET + POST
    AdminCveMisses,
    #[strum(props(path = "/admin/omni_cves", section = "Admin"))]
    AdminOmniCves,

    // Experimental
    #[strum(props(path = "/experimental/", section = "Experimental"))] // GET + POST
    Experimental,
    #[strum(props(path = "/experimental/turnover/maintainers", section = "Experimental"))]
    MaintainersTurnover,
    #[strum(props(path = "/experimental/distromap", section = "Experimental"))]
    Distromap,

    // Misc
    #[strum(props(path = "/link/*url"))]
    Link,
    #[strum(props(path = "/favicon.ico"))]
    Favicon,

    // API
    #[strum(props(path = "/api/v1/projects"))]
    ApiV1Projects,
    #[strum(props(path = "/api/v1/projects/:bound"))]
    ApiV1ProjectsBounded,
    #[strum(props(path = "/api/v1/repository/:repo/problems"))]
    ApiV1RepositoryProblems,
    #[strum(props(path = "/api/v1/maintainer/:maintainer/problems-for-repo/:repository_name"))]
    ApiV1MaintainerProblems,
    #[strum(props(path = "/api/experimental/distromap"))]
    ApiExperimentalDistromap,
    #[strum(props(path = "/api/experimental/updates"))]
    ApiExperimentalUpdates,

    // Graphs
    #[strum(props(path = "/graph/project/:project/releases.svg"))]
    GraphReleases,
    #[strum(props(path = "/graph/map_repo_size_fresh.svg"))]
    GraphMapRepoSizeFresh,
    #[strum(props(path = "/graph/map_repo_size_fresh_nonunique.svg"))]
    GraphMapRepoSizeFreshNonunique,
    #[strum(props(path = "/graph/map_repo_size_freshness.svg"))]
    GraphMapRepoSizeFreshness,
    #[strum(props(path = "/graph/repo/:repo/projects_total.svg"))]
    GraphRepoProjectsTotal,
    #[strum(props(path = "/graph/repo/:repo/projects_newest.svg"))]
    GraphRepoProjectsNewest,
    #[strum(props(path = "/graph/repo/:repo/projects_newest_percent.svg"))]
    GraphRepoProjectsNewestPercent,
    #[strum(props(path = "/graph/repo/:repo/projects_outdated.svg"))]
    GraphRepoProjectsOutdated,
    #[strum(props(path = "/graph/repo/:repo/projects_outdated_percent.svg"))]
    GraphRepoProjectsOutdatedPercent,
    #[strum(props(path = "/graph/repo/:repo/projects_unique.svg"))]
    GraphRepoProjectsUnique,
    #[strum(props(path = "/graph/repo/:repo/projects_unique_percent.svg"))]
    GraphRepoProjectsUniquePercent,
    #[strum(props(path = "/graph/repo/:repo/projects_problematic.svg"))]
    GraphRepoProjectsProblematic,
    #[strum(props(path = "/graph/repo/:repo/projects_problematic_percent.svg"))]
    GraphRepoProjectsProblematicPercent,
    #[strum(props(path = "/graph/repo/:repo/projects_vulnerable.svg"))]
    GraphRepoProjectsVulnerable,
    #[strum(props(path = "/graph/repo/:repo/projects_vulnerable_percent.svg"))]
    GraphRepoProjectsVulnerablePercent,
    #[strum(props(path = "/graph/repo/:repo/problems.svg"))]
    GraphRepoProblems,
    #[strum(props(path = "/graph/repo/:repo/problems_per_metapackage.svg"))]
    GraphRepoProblemsPerMetapackage,
    #[strum(props(path = "/graph/repo/:repo/maintainers.svg"))]
    GraphRepoMaintainers,
    #[strum(props(path = "/graph/repo/:repo/packages_per_maintainer.svg"))]
    GraphRepoPackagesPerMaintainer,
    #[strum(props(path = "/graph/total/packages.svg"))]
    GraphTotalPackages,
    #[strum(props(path = "/graph/total/projects.svg"))]
    GraphTotalProjects,
    #[strum(props(path = "/graph/total/maintainers.svg"))]
    GraphTotalMaintainers,
    #[strum(props(path = "/graph/total/problems.svg"))]
    GraphTotalProblems,

    // Opensearch
    #[strum(props(path = "/opensearch/project.xml"))]
    OpensearchProject,
    #[strum(props(path = "/opensearch/maintainer.xml"))]
    OpensearchMaintainer,

    // Badges
    #[strum(props(path = "/badge/versions-matrix.svg"))]
    BadgeVersionsMatrix,
    #[strum(props(path = "/badge/repository-big/:repo.svg"))]
    BadgeRepositoryBig,

    // Sitemaps
    #[strum(props(path = "/sitemaps/index.xml"))]
    SitemapIndex,
    #[strum(props(path = "/sitemaps/main.xml"))]
    SitemapMain,
    #[strum(props(path = "/sitemaps/repositories.xml"))]
    SitemapRepositories,
    #[strum(props(path = "/sitemaps/maintainers.xml"))]
    SitemapMaintainers,
    #[strum(props(path = "/sitemaps/projects_:int:page.xml"))]
    SitemapProjects,
}

#[derive(EnumString, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Admin,
    Docs,
    Experimental,
    Maintainers,
    News,
    Projects,
    Repositories,
    Security,
    Tools,
}

impl Endpoint {
    pub fn path(&self) -> &'static str {
        self.get_str("path")
            .expect("path should exist for the endpoint")
    }

    pub fn name(&self) -> &'static str {
        self.into()
    }

    pub fn is_section(&self, section: Section) -> bool {
        use std::str::FromStr as _;
        self.get_str("section")
            .is_some_and(|endpoint_section| Section::from_str(endpoint_section).unwrap() == section)
    }

    pub fn construct_url(&self, values: &HashMap<String, Value>) -> Result<String, Error> {
        let is_key_char = |c: char| c.is_lowercase() || c == '_';

        let mut rest = self.path();
        let mut res = String::new();

        loop {
            if let Some((prefix, key_and_rest)) = rest.split_once(":") {
                res += prefix;

                let (key, suffix) = key_and_rest.split_at(
                    key_and_rest
                        .find(|c| !is_key_char(c))
                        .unwrap_or(key_and_rest.len()),
                );

                match values
                    .get(key)
                    .ok_or(anyhow!("missing value for path placeholder \"{}\"", key))?
                {
                    Value::Number(n) => {
                        res += &n.to_string();
                    }
                    Value::String(s) => {
                        res += s;
                    }
                    _ => {
                        bail!("invalid value type for path placeholder \"{}\"", key);
                    }
                };
                rest = suffix;
            } else {
                return Ok(res + rest);
            }
        }
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
            "/badge/version-for-repo/:repository_name/:project_name.svg"
        );
    }

    #[test]
    fn test_name() {
        assert_eq!(Endpoint::BadgeVersionForRepo.name(), "BadgeVersionForRepo");
    }

    #[test]
    fn test_construct_url() {
        use serde_json::json;
        let repository_name: HashMap<String, Value> = [("repository_name".into(), json!("foo"))]
            .into_iter()
            .collect();
        let project_name: HashMap<String, Value> = [("project_name".into(), json!("bar"))]
            .into_iter()
            .collect();
        let project_and_repository_names = {
            let mut t = project_name.clone();
            t.extend(repository_name.clone().into_iter());
            t
        };

        assert_eq!(
            Endpoint::ApiV1Project.construct_url(&project_name).unwrap(),
            "/api/v1/project/bar"
        );

        assert_eq!(
            Endpoint::BadgeVersionForRepo
                .construct_url(&project_and_repository_names)
                .unwrap(),
            "/badge/version-for-repo/foo/bar.svg"
        );
        assert!(Endpoint::BadgeVersionForRepo
            .construct_url(&repository_name)
            .is_err());
    }
}
