// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages", "problems_data")
)]
async fn test_problems(pool: PgPool) {
    check_response!(
        pool,
        "/repository/nonexistent/problems",
        status NOT_FOUND,
    );
    check_response!(
        pool,
        "/repository/freebsd/problems",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",

        // contains maintainer column
        contains r#"<td class="text-center"><a href="/maintainer/johndoe"#,

        // each error kind is present
        contains "Homepage link <code>https://example.com/</code> is <",
        contains "Homepage link <code>https://example.com/</code> is a permanent redirect",
        contains "points to Google Code which was discontinued",
        contains "points to codeplex which was discontinued",
        contains "points to Gna which was discontinued",
        contains "points to CPAN which was discontinued",
        contains "was not found neither among known CVEs nor in NVD CPE dictionary",
        contains "CPE information is missing for this package",
        contains "Download link <code>https://example.com/</code> is <",
        contains "Download link <code>https://example.com/</code> is a permanent redirect",
        contains "needs a trailing slash added",

        contains_not "Unformatted problem of type",
    );

    check_response!(
        pool,
        "/maintainer/johndoe@example.com/problems-for-repo/nonexistent",
        status NOT_FOUND,
    );
    check_response!(
        pool,
        "/maintainer/johndoe@example.com/problems-for-repo/freebsd",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",

        // does not contain maintainer column, because maintainer is fixed
        contains_not r#"<td class="text-center"><a href="/maintainer/johndoe"#,

        // each error kind is present
        contains "Homepage link <code>https://example.com/</code> is <",
        contains "Homepage link <code>https://example.com/</code> is a permanent redirect",
        contains "points to Google Code which was discontinued",
        contains "points to codeplex which was discontinued",
        contains "points to Gna which was discontinued",
        contains "points to CPAN which was discontinued",
        contains "was not found neither among known CVEs nor in NVD CPE dictionary",
        contains "CPE information is missing for this package",
        contains "Download link <code>https://example.com/</code> is <",
        contains "Download link <code>https://example.com/</code> is a permanent redirect",
        contains "needs a trailing slash added",

        contains_not "Unformatted problem of type",
    );
}
