// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_projects_params_retained_by_the_form(pool: PgPool) {
    for url in ["/projects/", "/projects/foo/", "/projects/..foo/"] {
        check_response!(
            pool,
            url,
            status OK,
            content_type "text/html",
            html_ok "allow_empty_tags,warnings_fatal",
            contains_not "xsearchx",
            contains_not "xmaintainerx",
            contains_not "xcategoryx",
            line_matches_not "freebsd.*selected",
            line_matches_not "ubuntu_24.*selected",
            contains_not "970",
            contains_not "971",
            contains_not "972",
            contains_not "973",
            contains_not "974",
            contains_not "975",
            contains_not "976",
            contains_not "977",
            line_matches_not "newest.*checked",
            line_matches_not "outdated.*checked",
            line_matches_not "problematic.*checked",
            line_matches_not "vulnerable.*checked",
            line_matches_not "has_related.*checked",
        );

        check_response!(
            pool,
            &(url.to_string() + "?search=xsearchx&maintainer=xmaintainerx&category=xcategoryx&inrepo=freebsd&notinrepo=ubuntu_24&repos=970-971&families=972-973&repos_newest=974-975&families_newest=976-977&newest=1&outdated=1&problematic=1&vulnerable=1&has_related=1"),
            status OK,
            content_type "text/html",
            html_ok "allow_empty_tags,warnings_fatal",
            contains "xsearchx",
            contains "xmaintainerx",
            contains "xcategoryx",
            line_matches "freebsd.*selected",
            line_matches "ubuntu_24.*selected",
            contains "970",
            contains "971",
            contains "972",
            contains "973",
            contains "974",
            contains "975",
            contains "976",
            contains "977",
            line_matches "newest.*checked",
            line_matches "outdated.*checked",
            line_matches "problematic.*checked",
            line_matches "vulnerable.*checked",
            line_matches "has_related.*checked",
        );
    }
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_projects_pagination(pool: PgPool) {
    check_response!(pool,
        "/projects/",
        contains "pkg_barbar_",
        contains "pkg_foofoo_"
    );
    check_response!(pool,
        "/projects/pkg_foo/",
        contains_not "pkg_barbar_",
        contains "pkg_foofoo_"
    );
    check_response!(pool,
        "/projects/..pkg_foo/",
        contains "pkg_barbar_",
        contains_not "pkg_foofoo_"
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_projects_search(pool: PgPool) {
    check_response!(pool,
        "/projects/",
        contains "pkg_barbar_",
        contains "pkg_foofoo_"
    );
    check_response!(pool,
        "/projects/?search=bar",
        contains "pkg_barbar_",
        contains_not "pkg_foofoo_"
    );
    check_response!(pool,
        "/projects/?search=foo",
        contains_not "pkg_barbar_",
        contains "pkg_foofoo_"
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_projects_inrepo(pool: PgPool) {
    check_response!(pool,
        "/projects/",
        contains "pkg_12e",
        contains "pkg_24e",
        contains "pkg_1224e",
    );
    check_response!(pool,
        "/projects/?inrepo=ubuntu_12",
        contains "pkg_12e",
        contains_not "pkg_24e",
        contains "pkg_1224e",
    );
    check_response!(pool,
        "/projects/?notinrepo=ubuntu_12",
        contains_not "pkg_12e",
        contains "pkg_24e",
        contains_not "pkg_1224e",
    );
    check_response!(pool,
        "/projects/?inrepo=ubuntu_12&newest=1",
        contains "pkg_1224_newest_12",
        contains_not "pkg_1224_newest_24"
    );
    check_response!(pool,
        "/projects/?inrepo=ubuntu_12&outdated=1",
        contains_not "pkg_1224_newest_12",
        contains "pkg_1224_newest_24"
    );
}
