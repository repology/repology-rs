// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod top;

use std::sync::Arc;

use askama::Template;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use indoc::indoc;
use sqlx::FromRow;

use crate::endpoints::Endpoint;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;
use crate::views::projects::common::CategorizedDisplayVersions;
use crate::views::projects::common::PackageForListing;
use crate::views::projects::common::packages_to_categorized_display_versions_per_project;

use self::top::Top;

#[derive(FromRow)]
struct Repository {
    name: String,
    title: String,
    statistics_group: String,

    num_projects: i32,
    num_projects_unique: i32,
    num_projects_newest: i32,
    num_projects_comparable: i32,
    num_maintainers: i32,
}

struct TopRepository<'a> {
    name: &'a str,
    title: &'a str,
}

#[derive(Debug, FromRow)]
pub struct ImportantProject {
    pub effname: String,
    #[sqlx(try_from = "i16")]
    pub num_families: u32,
    pub has_related: bool,
}

pub struct ProjectListItem {
    pub project: ImportantProject,
    pub versions: CategorizedDisplayVersions,
}

#[derive(Template)]
#[template(path = "index.html")]
struct TemplateParams<'a> {
    ctx: TemplateContext,

    top_by_total: Vec<top::Item<&'a str, TopRepository<'a>>>,
    top_by_nonunique: Vec<top::Item<&'a str, TopRepository<'a>>>,
    top_by_maintainers: Vec<top::Item<&'a str, TopRepository<'a>>>,
    top_by_ppm: Vec<top::Item<&'a str, TopRepository<'a>>>,
    top_by_newest: Vec<top::Item<&'a str, TopRepository<'a>>>,
    top_by_pnewest: Vec<top::Item<&'a str, TopRepository<'a>>>,

    projects_list: Vec<ProjectListItem>,
}

const IMPORTANT_PROJECTS: &[&str] = &[
    "apache24",
    "awesome",
    "bash",
    "binutils",
    "bison",
    "blender",
    "boost",
    "bzip2",
    "chromium",
    "claws-mail",
    "cmake",
    "cppcheck",
    "cups",
    "curl",
    "darktable",
    "dia",
    "djvulibre",
    "dosbox",
    "dovecot",
    "doxygen",
    "dvd+rw-tools",
    "emacs",
    "exim",
    "ffmpeg",
    "firefox",
    "flex",
    "fluxbox",
    "freecad",
    "freetype",
    "gcc",
    "gdb",
    "geeqie",
    "gimp",
    "git",
    "gnupg",
    "go",
    "graphviz",
    "grub",
    "icewm",
    "imagemagick",
    "inkscape",
    "irssi",
    "kodi",
    "lame",
    "lftp",
    "libreoffice",
    "libressl",
    "lighttpd",
    "links",
    "llvm",
    "mariadb",
    "maxima",
    "mc",
    "memcached",
    "mercurial",
    "mesa",
    "mplayer",
    "mutt",
    "mysql",
    "nginx",
    "nmap",
    "octave",
    "openbox",
    "openssh",
    "openssl",
    "openttf",
    "openvpn",
    "p7zip",
    "perl",
    "pidgin",
    "postfix",
    "postgresql",
    "privoxy",
    "procmail",
    "python",
    "qemu",
    "rdesktop",
    "redis",
    "rrdtool",
    "rsync",
    "rtorrent",
    "rxvt-unicode",
    "samba",
    "sane-backends",
    "scons",
    "screen",
    "scribus",
    "scummvm",
    "sdl2",
    "smartmontools",
    "sqlite",
    "squid",
    "subversion",
    "sudo",
    "sumversion",
    "thunderbird",
    "tigervnc",
    "tmux",
    "tor",
    "valgrind",
    "vim",
    "virtualbox",
    "vlc",
    "vsftpd",
    "wayland",
    "wesnoth",
    "wget",
    "wine",
    "wireshark",
    "xorg-server",
    "youtube-dl",
    "zsh",
];

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all))]
pub async fn index(State(state): State<Arc<AppState>>) -> EndpointResult {
    let ctx = TemplateContext::new_without_params(Endpoint::Index);

    let mut top_by_total = Top::<&str, TopRepository>::new(
        crate::constants::REPOSITORY_TOP_SIZE,
        top::Precedence::Greatest,
    );
    let mut top_by_nonunique = Top::<&str, TopRepository>::new(
        crate::constants::REPOSITORY_TOP_SIZE,
        top::Precedence::Greatest,
    );
    let mut top_by_maintainers = Top::<&str, TopRepository>::new(
        crate::constants::REPOSITORY_TOP_SIZE,
        top::Precedence::Greatest,
    );
    let mut top_by_ppm = Top::<&str, TopRepository>::new(
        crate::constants::REPOSITORY_TOP_SIZE,
        top::Precedence::Lowest,
    );
    let mut top_by_newest = Top::<&str, TopRepository>::new(
        crate::constants::REPOSITORY_TOP_SIZE,
        top::Precedence::Greatest,
    );
    let mut top_by_pnewest = Top::<&str, TopRepository>::new(
        crate::constants::REPOSITORY_TOP_SIZE,
        top::Precedence::Greatest,
    );

    let repositories: Vec<Repository> = sqlx::query_as(indoc! {r#"
        SELECT
            name,
            "desc" AS title,
            coalesce(metadata->>'statsgroup', "desc") AS statistics_group,
            num_metapackages AS num_projects,
            num_metapackages_unique AS num_projects_unique,
            num_metapackages_newest AS num_projects_newest,
            num_metapackages_comparable AS num_projects_comparable,
            num_maintainers AS num_maintainers
        FROM repositories
        WHERE state = 'active' AND metadata->>'type' = 'repository'
        ORDER BY sortname
    "#})
    .fetch_all(&state.pool)
    .await?;

    for repository in &repositories {
        if repository.num_projects as usize >= crate::constants::MIN_REPOSITORY_SIZE_FOR_TOP {
            top_by_total.add(
                &repository.statistics_group,
                TopRepository {
                    name: &repository.name,
                    title: &repository.title,
                },
                repository.num_projects as f64,
            );
            top_by_nonunique.add(
                &repository.statistics_group,
                TopRepository {
                    name: &repository.name,
                    title: &repository.title,
                },
                (repository.num_projects - repository.num_projects_unique) as f64,
            );
            top_by_maintainers.add(
                &repository.statistics_group,
                TopRepository {
                    name: &repository.name,
                    title: &repository.title,
                },
                repository.num_maintainers as f64,
            );
            top_by_newest.add(
                &repository.statistics_group,
                TopRepository {
                    name: &repository.name,
                    title: &repository.title,
                },
                repository.num_projects_newest as f64,
            );

            if repository.num_projects_comparable > 0 {
                top_by_pnewest.add(
                    &repository.statistics_group,
                    TopRepository {
                        name: &repository.name,
                        title: &repository.title,
                    },
                    100.0 * repository.num_projects_newest as f64
                        / repository.num_projects_comparable as f64,
                );
            }
            if repository.num_maintainers > 0 {
                top_by_ppm.add(
                    &repository.statistics_group,
                    TopRepository {
                        name: &repository.name,
                        title: &repository.title,
                    },
                    repository.num_projects as f64 / repository.num_maintainers as f64,
                );
            }
        }
    }

    let projects: Vec<ImportantProject> = sqlx::query_as(indoc! {"
        SELECT
            effname,
            num_families,
            has_related
        FROM metapackages
        WHERE effname = ANY($1)
        ORDER BY effname
    "})
    .bind(&IMPORTANT_PROJECTS)
    .fetch_all(&state.pool)
    .await?;

    // XXX: we don't need to fetch repo and maintainers here as these are never used
    // in packages_to_categorized_display_versions_per_project(..., None, None). In
    // fact, we need to add specialization for focusless case.
    let packages: Vec<PackageForListing> = sqlx::query_as(indoc! {"
        SELECT
            '' AS repo,
            family,
            visiblename,
            effname,
            version,
            versionclass AS status,
            flags,
            '{}'::text[] AS maintainers
        FROM packages
        WHERE effname = ANY($1)
    "})
    .bind(&IMPORTANT_PROJECTS)
    .fetch_all(&state.pool)
    .await?;

    let mut versions_per_project =
        packages_to_categorized_display_versions_per_project(&packages, None, None);

    let projects_list: Vec<_> = projects
        .into_iter()
        .map(|project| {
            let versions = versions_per_project
                .remove(&project.effname)
                .unwrap_or_default();
            ProjectListItem { project, versions }
        })
        .collect();

    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            top_by_total: top_by_total.get().collect(),

            top_by_nonunique: top_by_nonunique.get().collect(),
            top_by_maintainers: top_by_maintainers.get().collect(),
            top_by_ppm: top_by_ppm.get().collect(),
            top_by_newest: top_by_newest.get().collect(),
            top_by_pnewest: top_by_pnewest.get().collect(),
            projects_list,
        }
        .render()?,
    )
        .into_response())
}
