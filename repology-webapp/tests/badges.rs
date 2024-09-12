// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::{check_code, check_svg};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_data"))]
async fn test_badge_tiny_repos(pool: PgPool) {
    check_code!(pool, "/badge/tiny-repos/nonexistent", NOT_FOUND);
    check_svg!(
        pool,
        "/badge/tiny-repos/nonexistent.svg",
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 4_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "in repositories",
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "0",
    );
    check_svg!(pool, "/badge/tiny-repos/zsh.svg",
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 4_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "in repositories",
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "3",
    );

    // caption flags
    check_svg!(
        pool,
        "/badge/tiny-repos/zsh.svg?header=Repository+Count",
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "Repository Count",
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "3",
    );
    check_svg!(
        pool,
        "/badge/tiny-repos/zsh.svg?header=",
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 2_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "3",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_data"))]
async fn test_badge_version_for_repo(pool: PgPool) {
    check_code!(pool, "/badge/version-for-repo/freebsd/zsh", NOT_FOUND);
    check_code!(pool, "/badge/version-for-repo/badrepo/zsh.svg", NOT_FOUND);
    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg",
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 4_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "FreeBSD port",
        @"count(//svg:g[1]/svg:rect[@fill='#4c1'])" == 1_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "1.1",
    );

    // minversion_flag
    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg?minversion=1.2",
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "FreeBSD port",
        @"count(//svg:g[1]/svg:rect[@fill='#e00000'])" == 1_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "1.1",
    );

    // caption flags
    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg?header=fbsd+ver",
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "fbsd ver",
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "1.1",
    );
    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg?header=",
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 2_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "1.1",
    );

    check_svg!(
        pool,
        "/badge/version-for-repo/freebsd/unpackaged.svg",
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "FreeBSD port",
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "-",
    );
    check_svg!(
        pool,
        "/badge/version-for-repo/ubuntu_24/zsh.svg",
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "Ubuntu 24 package",
        @"count(//svg:g[1]/svg:rect[@fill='#e05d44'])" == 1_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "1.0",
    );
    check_svg!(
        pool,
        "/badge/version-for-repo/ubuntu_24/zsh.svg?allow_ignored=1",
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "Ubuntu 24 package",
        @"count(//svg:g[1]/svg:rect[@fill='#9f9f9f'])" == 1_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "1.2",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_data"))]
async fn test_badge_vertical_allrepos(pool: PgPool) {
    check_code!(pool, "/badge/vertical-allrepos/zsh", NOT_FOUND);

    // caption
    check_svg!(
        pool,
        "/badge/vertical-allrepos/unpackaged.svg",
        @"string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" == "No known packages",
        @"count(//svg:g[1]/svg:g[@font-size=11]/svg:text)" == 0_f64,
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/unpackaged.svg?header=Packages",
        @"string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" == "Packages",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/unpackaged.svg?header=",
        @"count(//svg:g[1]/svg:g[@font-size=15]/svg:text)" == 0_f64,
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg",
        @"string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" == "Packaging status",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?header=Packages",
        @"string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" == "Packages",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?header=",
        @"count(//svg:g[1]/svg:g[@font-size=15]/svg:text)" == 0_f64,
    );

    // version flags
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" == "FreeBSD",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[3])" == "1.1",
        @"string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])" == "freshcode.club",
        @"string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[3])" == "1.0",
        @"string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])" == "Ubuntu 12",
        @"string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[3])" == "0.9",
        @"string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])" == "Ubuntu 24",
        @"string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[3])" == "1.0",
        @"string(//svg:g[1]/svg:rect[2]/@fill)" == "#4c1",
        @"string(//svg:g[1]/svg:rect[4]/@fill)" == "#e05d44",
        @"string(//svg:g[1]/svg:rect[6]/@fill)" == "#e05d44",
        @"string(//svg:g[1]/svg:rect[8]/@fill)" == "#e05d44",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?allow_ignored=1",
        @"string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])" == "Ubuntu 24",
        @"string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[3])" == "1.2",
        @"string(//svg:g[1]/svg:rect[8]/@fill)" == "#9f9f9f",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?minversion=1.0",
        @"string(//svg:g[1]/svg:rect[2]/@fill)" == "#4c1",
        @"string(//svg:g[1]/svg:rect[4]/@fill)" == "#e05d44",
        @"string(//svg:g[1]/svg:rect[6]/@fill)" == "#e00000",
        @"string(//svg:g[1]/svg:rect[8]/@fill)" == "#e05d44",
    );

    // repository filters
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" == "FreeBSD",
        @"string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])" == "freshcode.club",
        @"string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])" == "Ubuntu 12",
        @"string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])" == "Ubuntu 24",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?exclude_unsupported=1",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" == "FreeBSD",
        @"string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])" == "freshcode.club",
        @"string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])" == "Ubuntu 24",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?exclude_sources=site",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" == "FreeBSD",
        @"string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])" == "Ubuntu 12",
        @"string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])" == "Ubuntu 24",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" == "freshcode.club",
    );
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository,site",
        @"count(//svg:g[1]/svg:g[@font-size=11]/svg:text)" == 0_f64,
    );

    // columns
    check_svg!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?columns=4",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" == "FreeBSD",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[3])" == "1.1",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[5])" == "freshcode.club",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[7])" == "1.0",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[9])" == "Ubuntu 12",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[11])" == "0.9",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[13])" == "Ubuntu 24",
        @"string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[15])" == "1.0",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("badge_versions_data")
)]
async fn test_badge_latest_versions(pool: PgPool) {
    check_code!(pool, "/badge/latest-versions/nonexistent", NOT_FOUND);
    check_svg!(
        pool,
        "/badge/latest-versions/nonexistent.svg",
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 4_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "latest packaged version",
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "-",
    );
    check_svg!(
        pool,
        "/badge/latest-versions/zsh.svg",
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 4_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "latest packaged versions",
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "3.0, 1.0.0, 1_0_0, 1.0",
    );
    check_svg!(
        pool,
        "/badge/latest-versions/bash.svg",
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 4_f64,
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "latest packaged version",
        @"string(//svg:g[1]/svg:g[1]/svg:text[3])" == "1.0",
    );

    // caption flags
    check_svg!(
        pool,
        "/badge/latest-versions/zsh.svg?header=VERSIONS",
        @"string(//svg:g[1]/svg:g[1]/svg:text[1])" == "VERSIONS",
    );
    check_svg!(
        pool,
        "/badge/latest-versions/zsh.svg?header=",
        @"count(//svg:g[1]/svg:g[1]/svg:text)" == 2_f64,
    );
}
