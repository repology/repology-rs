// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use serde_repr::Deserialize_repr;

/// Type of a web link which can be attached to a package.
///
/// Possible values fall into three primary categories:
/// - `Upstream*` are links to resources dedicated to the project and maintained by
///   its authors. It could be a homepage (such as `https://cmake.org/`) or its
///   main repository (such as `https://github.com/Kitware/CMake`).
/// - `Project*` (TODO: Rename to `Registry*`) are links refer to project pages on official
///   package registries, such as PyPI (Python), crates.io (Rust), rubygems (Ruby), etc.
/// - `Package*` are links related to downstream packages.
///
/// Some types may have `*Raw` counterpart, such as `PackageRecipe` and `PackageRecipeRaw`,
/// in which case `PackageRecipe` is used for pages dedicated for human consumption (such
/// as HTML page with syntax highlighting), and `PackageRecipeRaw` is plain text more suitable
/// for machine consumption and parsing.
///
/// Note that a single link may belong to multiple categories (such as
/// `https://flask.palletsprojects.com` can be both `UpstreamHomepage` and `UpstreamDocumentation`
/// and `https://github.com/pallets/flask` can be both `UpstreamHomepage` and `UpstreamRepository`),
/// and these values are in fact used to differentiate *link sources* rather than links themselves.
#[derive(Debug, PartialEq, Eq, sqlx::Type, PartialOrd, Ord, Hash, Deserialize_repr, Copy, Clone)]
#[repr(i8)]
pub enum LinkType {
    /// Projects official home page maintained by its authors.
    ///
    /// This is not required to be a dedicated web site, projects official repository
    /// servers as a home page just as well.
    ///
    /// Examples:
    /// - `https://cmake.org/`
    /// - `https://github.com/Kitware/CMake`
    UpstreamHomepage = 0,

    /// Upstream download.
    ///
    /// Source code archive, prebuilt binary, or an installer.
    ///
    /// Examples:
    /// - `https://github.com/Kitware/CMake/releases/download/v3.31.0-rc3/cmake-3.31.0-rc3-macos10.10-universal.dmg`
    /// - `http://ftp.mozilla.org/pub/firefox/releases/132.0/linux-x86_64/en-US/firefox-132.0.tar.bz2`
    UpstreamDownload = 1,

    /// Official source code repository.
    ///
    /// Can be both repository front page or an URL suitable for cloning, as most sources
    /// do not differentiate these. Thankfully, these kinds of URLs may be used interchangeably
    /// in most cases.
    ///
    /// Examples:
    /// - `https://github.com/Kitware/CMake`
    /// - `https://github.com/Kitware/CMake.git`
    UpstreamRepository = 2,

    /// Upstream issue tracker.
    ///
    /// Examples:
    /// - `https://gitlab.kitware.com/cmake/cmake/-/issues`
    /// - `https://bugzilla.mozilla.org/home`
    UpstreamIssueTracker = 3,

    /// Project page on a well known module registry.
    ///
    /// Examples:
    /// - `https://pypi.org/project/Flask/`
    /// - `https://crates.io/crates/axum`
    // TODO: Rename to RegistryPage
    ProjectHomepage = 4,

    /// Page dedicated to the package in downstream repository.
    ///
    /// Examples:
    /// - `https://packages.debian.org/bookworm/zsh`
    /// - `https://www.freshports.org/shells/zsh`
    PackageHomepage = 5,

    /// Download location for downstream package.
    ///
    /// Examples:
    /// - `http://ftp.us.debian.org/debian/pool/main/z/zsh/zsh_5.9-4+b5_amd64.deb`
    /// - `https://fr2.rpmfind.net/linux/fedora/linux/releases/41/Everything/x86_64/os/Packages/p/perl-5.40.0-511.fc41.x86_64.rpm`
    PackageDownload = 6,

    /// Source tree for downstream package.
    ///
    /// This usually points to a repository, or a directory where files needed
    /// to build the package reside. These files may include recipe in either
    /// format, manifests, patches, checksum files etc.
    ///
    /// Example:
    /// - `https://cgit.freebsd.org/ports/tree/www/firefox/`
    PackageSources = 7,

    /// Issue tracker dedicated to downstream package.
    ///
    /// Since is most cases there's single issue tracker for downstream package
    /// repository, these links usually point to a search page on that tracker,
    /// which reliably lists issues related to a package.
    ///
    /// Examples:
    /// - `https://bugs.debian.org/firefox-esr`
    /// - `https://bugs.freebsd.org/bugzilla/buglist.cgi?bug_status=__open__&bug_status=__closed__&f0=short_desc&list_id=753328&o0=substring&query_format=advanced&v0=www%2Ffirefox`
    PackageIssueTracker = 8,

    /// Package recipe or build script, pretty formatted.
    ///
    /// Makefile, ebuild, PKGBUILD, .spec or whatever format downstream uses to
    /// describe steps which produce the package.
    ///
    /// Unlike `PackageRecipeRaw`, this is intended for human consumption, e.g.
    /// it's usually a HTML page with syntax highlighting.
    ///
    /// Examples:
    /// - `https://src.fedoraproject.org/rpms/firefox/blob/rawhide/f/firefox.spec`
    PackageRecipe = 9,

    /// Package recipe or build script, plain text.
    ///
    /// Same as `PackageRecipe`, but intended for machine consumption, e.g. usually
    /// plain text.
    ///
    /// Examples:
    /// - `https://src.fedoraproject.org/rpms/firefox/raw/rawhide/f/firefox.spec`
    PackageRecipeRaw = 10,

    /// Patch which is applied during package building, pretty formatted.
    ///
    /// Unlike `PackagePatchRaw`, this is intended for human consumption, e.g.
    /// it's usually a HTML page with syntax highlighting.
    ///
    /// Examples:
    /// - `https://cgit.freebsd.org/ports/tree/www/firefox/files/patch-bug1874059?h=2024Q4`
    PackagePatch = 11,

    /// Patch which is applied during package building, plain text.
    ///
    /// Same as `PackageRecipe`, but intended for machine consumption, e.g. usually
    /// plain text.
    ///
    /// Examples:
    /// - `https://cgit.freebsd.org/ports/plain/www/firefox/files/patch-bug1874059?h=2024Q4`
    PackagePatchRaw = 12,

    /// Log of a downstream package build process, pretty formatted.
    ///
    /// Unlike `PackageBuildLogRaw`, this is intended for human consumption, e.g.
    /// it's usually a HTML page with syntax highlighting.
    PackageBuildLog = 13,

    /// Log of a downstream package build process, plain text.
    ///
    /// Same as `PackageBuildLog`, but intended for machine consumption, e.g. usually
    /// plain text.
    PackageBuildLogRaw = 14,

    /// Downstream tool which checks for new project versions.
    PackageNewVersionChecker = 15,

    /// Official upstream documentation.
    ///
    /// Examples:
    /// - `https://flask.palletsprojects.com/en/stable/`
    /// - `https://docs.rs/axum/latest/axum/`
    UpstreamDocumentation = 16,

    /// Upstream changelog.
    ///
    /// Either formatted or raw format.
    ///
    /// Examples:
    /// - `https://flask.palletsprojects.com/en/stable/`
    /// - `https://docs.rs/axum/latest/axum/`
    // TODO: Split into UpstreamChangelog and UpstreamChangelogRaw
    UpstreamChangelog = 17,

    /// Download on a well known module registry.
    ///
    /// Example:
    /// - `https://files.pythonhosted.org/packages/41/e1/d104c83026f8d35dfd2c261df7d64738341067526406b40190bc063e829a/flask-3.0.3.tar.gz`
    // TODO: Rename to RegistryDownload
    ProjectDownload = 18,

    // TODO: finish documentation
    UpstreamDonation = 19, // XXX: to be used sparingly not to provide obsolete funding info
    UpstreamDiscussion = 20,
    UpstreamCoverage = 21,
    UpstreamCi = 22,
    UpstreamWiki = 23,
    PackageStatistics = 25,
    PackageBuildStatus = 26,

    /// Upstream page with links to build logs.
    ///
    /// Usually used if there's no way to extract direct links to build logs.
    PackageBuildLogs = 27,

    /// Upstream page with links to downloads.
    ///
    /// Usually used if there's no way to extract direct links to downloads.
    UpstreamDownloadPage = 28,

    /// Unclassified link type.
    ///
    /// This should not normally be used - if there's another link type, we
    /// can create a category for it right away.
    // TODO: Remove, as soon as parsers are rewritten in Rust.
    Other = 99,
}

impl LinkType {
    pub fn is_raw(&self) -> bool {
        use LinkType::*;
        match self {
            PackageRecipeRaw | PackagePatchRaw | PackageBuildLogRaw => true,
            _ => false,
        }
    }

    pub fn raw_counterpart(&self) -> Option<Self> {
        use LinkType::*;
        match self {
            PackageBuildLog => Some(PackageBuildLogRaw),
            PackagePatch => Some(PackagePatchRaw),
            PackageRecipe => Some(PackageRecipeRaw),
            _ => None,
        }
    }
}
