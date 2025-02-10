// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_badge_version_for_repo(pool: PgPool) {
    check_response!(pool, "/badge/version-for-repo/freebsd/zsh", status NOT_FOUND);
    check_response!(pool, "/badge/version-for-repo/badrepo/zsh.svg", status NOT_FOUND);
    check_response!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[1]/svg:text)" 4_f64,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "FreeBSD port",
        svg_xpath "count(//svg:g[1]/svg:rect[@fill='#4c1'])" 1_f64,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[3])" "1.1",
    );

    // minversion_flag
    check_response!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg?minversion=1.2",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "FreeBSD port",
        svg_xpath "count(//svg:g[1]/svg:rect[@fill='#e00000'])" 1_f64,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[3])" "1.1",
    );

    // caption flags
    check_response!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg?header=fbsd+ver",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "fbsd ver",
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[3])" "1.1",
    );
    check_response!(
        pool,
        "/badge/version-for-repo/freebsd/zsh.svg?header=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[1]/svg:text)" 2_f64,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "1.1",
    );

    check_response!(
        pool,
        "/badge/version-for-repo/freebsd/unpackaged.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "FreeBSD port",
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[3])" "-",
    );
    check_response!(
        pool,
        "/badge/version-for-repo/ubuntu_24/zsh.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "Ubuntu 24 package",
        svg_xpath "count(//svg:g[1]/svg:rect[@fill='#e05d44'])" 1_f64,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[3])" "1.0",
    );
    check_response!(
        pool,
        "/badge/version-for-repo/ubuntu_24/zsh.svg?allow_ignored=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "Ubuntu 24 package",
        svg_xpath "count(//svg:g[1]/svg:rect[@fill='#9f9f9f'])" 1_f64,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[3])" "1.2",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_badge_vertical_allrepos(pool: PgPool) {
    check_response!(pool, "/badge/vertical-allrepos/zsh", status NOT_FOUND);

    // caption
    check_response!(
        pool,
        "/badge/vertical-allrepos/unpackaged.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" "No known packages",
        svg_xpath "count(//svg:g[1]/svg:g[@font-size=11]/svg:text)" 0_f64,
    );
    check_response!(
        pool,
        "/badge/vertical-allrepos/unpackaged.svg?header=Packages",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" "Packages",
    );
    check_response!(
        pool,
        "/badge/vertical-allrepos/unpackaged.svg?header=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[@font-size=15]/svg:text)" 0_f64,
    );
    check_response!(
        pool,
        "/badge/vertical-allrepos/zsh.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" "Packaging status",
    );
    check_response!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?header=Packages",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" "Packages",
    );
    check_response!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?header=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[@font-size=15]/svg:text)" 0_f64,
    );

    // version flags
    check_response!(
        pool,
        "/badge/vertical-allrepos/zsh.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" "FreeBSD",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[3])" "1.1",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])" "freshcode.club",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[3])" "1.0",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])" "Ubuntu 12",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[3])" "0.9",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])" "Ubuntu 24",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[3])" "1.0",
        svg_xpath "string(//svg:g[1]/svg:rect[2]/@fill)" "#4c1",
        svg_xpath "string(//svg:g[1]/svg:rect[4]/@fill)" "#e05d44",
        svg_xpath "string(//svg:g[1]/svg:rect[6]/@fill)" "#e05d44",
        svg_xpath "string(//svg:g[1]/svg:rect[8]/@fill)" "#e05d44",
    );
    check_response!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?allow_ignored=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])" "Ubuntu 24",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[3])" "1.2",
        svg_xpath "string(//svg:g[1]/svg:rect[8]/@fill)" "#9f9f9f",
    );
    check_response!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?minversion=1.0",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:rect[2]/@fill)" "#4c1",
        svg_xpath "string(//svg:g[1]/svg:rect[4]/@fill)" "#e05d44",
        svg_xpath "string(//svg:g[1]/svg:rect[6]/@fill)" "#e00000",
        svg_xpath "string(//svg:g[1]/svg:rect[8]/@fill)" "#e05d44",
    );

    // repository filters
    check_response!(
        pool,
        "/badge/vertical-allrepos/zsh.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" "FreeBSD",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])" "freshcode.club",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])" "Ubuntu 12",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])" "Ubuntu 24",
    );
    check_response!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?exclude_unsupported=1",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" "FreeBSD",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])" "freshcode.club",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])" "Ubuntu 24",
    );
    check_response!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?exclude_sources=site",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" "FreeBSD",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])" "Ubuntu 12",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])" "Ubuntu 24",
    );
    check_response!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" "freshcode.club",
    );
    check_response!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository,site",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[@font-size=11]/svg:text)" 0_f64,
    );

    // columns
    check_response!(
        pool,
        "/badge/vertical-allrepos/zsh.svg?columns=4",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" "FreeBSD",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[3])" "1.1",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[5])" "freshcode.club",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[7])" "1.0",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[9])" "Ubuntu 12",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[11])" "0.9",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[13])" "Ubuntu 24",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[15])" "1.0",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("badge_versions_data"))]
async fn test_badge_latest_versions(pool: PgPool) {
    check_response!(pool, "/badge/latest-versions/nonexistent", status NOT_FOUND);
    check_response!(
        pool,
        "/badge/latest-versions/nonexistent.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[1]/svg:text)" 4_f64,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "latest packaged version",
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[3])" "-",
    );
    check_response!(
        pool,
        "/badge/latest-versions/zsh.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[1]/svg:text)" 4_f64,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "latest packaged versions",
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[3])" "3.0, 1.0.0, 1_0_0, 1.0",
    );
    check_response!(
        pool,
        "/badge/latest-versions/bash.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[1]/svg:text)" 4_f64,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "latest packaged version",
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[3])" "1.0",
    );

    // caption flags
    check_response!(
        pool,
        "/badge/latest-versions/zsh.svg?header=VERSIONS",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[1]/svg:text[1])" "VERSIONS",
    );
    check_response!(
        pool,
        "/badge/latest-versions/zsh.svg?header=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[1]/svg:text)" 2_f64,
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "badge_repository_big_data"))]
async fn test_badge_repositry_big(pool: PgPool) {
    check_response!(pool, "/badge/repository-big/nonexistent", status NOT_FOUND);

    // repository which never existed or which is not active (i.e. was removed) are handled the same way
    check_response!(
        pool,
        "/badge/repository-big/nonexistent.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" "Repository status",
        svg_xpath "count(//svg:g[1]/svg:g[@font-size=11]/svg:text)" 2_f64,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" "Repository not known or was removed",
    );
    check_response!(
        pool,
        "/badge/repository-big/ubuntu_10.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" "Repository status",
        svg_xpath "count(//svg:g[1]/svg:g[@font-size=11]/svg:text)" 2_f64,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" "Repository not known or was removed",
    );

    // active repository which does not have any packages
    check_response!(
        pool,
        "/badge/repository-big/freshcode.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" "Repository status",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" "Projects total",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[3])" "0",
    );

    // complete repository with packages
    check_response!(
        pool,
        "/badge/repository-big/freebsd.svg",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" "Repository status",
        svg_xpath "count(//svg:g[1]/svg:g[@font-size=11]/svg:text)" 32_f64,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[1])" "Projects total",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][1]/svg:text[3])" "10",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[1])" "Up to date",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[3])" "1",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][2]/svg:text[5])" "20.00%",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[1])" "Outdated",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[3])" "2",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][3]/svg:text[5])" "40.00%",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[1])" "Vulnerable",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[3])" "3",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][4]/svg:text[5])" "30.00%",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][5]/svg:text[1])" "Bad versions",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][5]/svg:text[3])" "4",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][5]/svg:text[5])" "40.00%",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][6]/svg:text[1])" "Maintainers",
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=11][6]/svg:text[3])" "7",
    );

    // title tweaks
    check_response!(
        pool,
        "/badge/repository-big/freebsd.svg?header=FreeBSD",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[@font-size=15])" 1_f64,
        svg_xpath "string(//svg:g[1]/svg:g[@font-size=15]/svg:text[1])" "FreeBSD",
    );
    check_response!(
        pool,
        "/badge/repository-big/freebsd.svg?header=",
        status OK,
        content_type IMAGE_SVG,
        svg_xpath "count(//svg:g[1]/svg:g[@font-size=15])" 0_f64,
    );
}
