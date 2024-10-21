// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_data"))]
async fn test_projects_params_retained_by_the_form(pool: PgPool) {
    check_response!(
        pool,
        "/projects/?search=xsearchx&maintainer=xmaintainerx&category=xcategoryx&inrepo=freebsd&notinrepo=ubuntu_24_04&repos=970-971&families=972-973&repos_newest=974-975&families_newest=976-977&newest=1&outdated=1&problematic=1&vulnerable=1&has_related=1",
        status OK,
        content_type "text/html",
        contains "xsearchx",
        contains "xmaintainerx",
        contains "xcategoryx",
        contains "freebsd",
        contains "ubuntu_24_04",
        contains "970",
        contains "971",
        contains "972",
        contains "973",
        contains "974",
        contains "975",
        contains "976",
        contains "977",
        line_matches "newest.*checked=\"checked\"",
        line_matches "outdated.*checked=\"checked\"",
        line_matches "problematic.*checked=\"checked\"",
        line_matches "vulnerable.*checked=\"checked\"",
        line_matches "has_related.*checked=\"checked\"",
    );
}
