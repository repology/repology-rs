// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "repository_feed_data"))]
async fn test_repository_feed(pool: PgPool) {
    check_response!(
        pool,
        "/repository/nonexistent/feed",
        status NOT_FOUND,
    );
    check_response!(
        pool,
        "/repository/freebsd/feed",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",

        contains ">111<",
        contains ">222<",
        contains ">333<",
        contains ">444<",
        contains ">555<",
        contains ">666<",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "repository_feed_data"))]
async fn test_repository_feed_atom(pool: PgPool) {
    check_response!(
        pool,
        "/repository/nonexistent/feed/atom",
        status NOT_FOUND,
    );
    check_response!(
        pool,
        "/repository/freebsd/feed/atom",
        status OK,
        content_type "application/atom+xml",

        xpath "count(/*[local-name()='feed']/*[local-name()='entry'])" 6_f64,
        xpath "string(/*[local-name()='feed']/*[local-name()='entry'][1]/*[local-name()='title'])" "zsh is no longer tracked",
        xpath "string(/*[local-name()='feed']/*[local-name()='entry'][2]/*[local-name()='title'])" "zsh 666 is ignored",
        xpath "string(/*[local-name()='feed']/*[local-name()='entry'][3]/*[local-name()='title'])" "zsh 222 is outdated by 333, 444",
        xpath "string(/*[local-name()='feed']/*[local-name()='entry'][4]/*[local-name()='title'])" "zsh 555 is outdated",
        xpath "string(/*[local-name()='feed']/*[local-name()='entry'][5]/*[local-name()='title'])" "zsh 111 is up to date",
        xpath "string(/*[local-name()='feed']/*[local-name()='entry'][6]/*[local-name()='title'])" "zsh is now tracked",
        contains_not "\n\n",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "common_maintainers", "maintainer_feed_data"))]
async fn test_maintainer_repo_feed(pool: PgPool) {
    check_response!(
        pool,
        "/maintainer/nonexistent@example.com/feed-for-repo/freebsd",
        // we don't currently check for maintainer existence, so it's just an empty feed
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
    );
    check_response!(
        pool,
        "/maintainer/johndoe@example.com/feed-for-repo/nonexistent",
        status NOT_FOUND,
    );
    check_response!(
        pool,
        "/maintainer/johndoe@example.com/feed-for-repo/freebsd",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",

        contains ">111<",
        contains ">222<",
        contains ">333<",
        contains ">444<",
        contains ">555<",
        contains ">666<",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "common_maintainers", "maintainer_feed_data"))]
async fn test_maintainer_repo_feed_atom(pool: PgPool) {
    check_response!(
        pool,
        "/maintainer/nonexistent@example.com/feed-for-repo/freebsd/atom",
        // we don't currently check for maintainer existence, so it's just an empty feed
        status OK,
        content_type "application/atom+xml",
        xpath "count(/*[local-name()='feed']/*[local-name()='entry'])" 0_f64,
        contains_not "\n\n",
    );
    check_response!(
        pool,
        "/maintainer/johndoe@example.com/feed-for-repo/nonexistent/atom",
        status NOT_FOUND,
    );
    check_response!(
        pool,
        "/maintainer/johndoe@example.com/feed-for-repo/freebsd/atom",
        status OK,
        content_type "application/atom+xml",

        xpath "count(/*[local-name()='feed']/*[local-name()='entry'])" 6_f64,
        xpath "string(/*[local-name()='feed']/*[local-name()='entry'][1]/*[local-name()='title'])" "zsh is no longer tracked",
        xpath "string(/*[local-name()='feed']/*[local-name()='entry'][2]/*[local-name()='title'])" "zsh 666 is ignored",
        xpath "string(/*[local-name()='feed']/*[local-name()='entry'][3]/*[local-name()='title'])" "zsh 222 is outdated by 333, 444",
        xpath "string(/*[local-name()='feed']/*[local-name()='entry'][4]/*[local-name()='title'])" "zsh 555 is outdated",
        xpath "string(/*[local-name()='feed']/*[local-name()='entry'][5]/*[local-name()='title'])" "zsh 111 is up to date",
        xpath "string(/*[local-name()='feed']/*[local-name()='entry'][6]/*[local-name()='title'])" "zsh is now tracked",
        contains_not "\n\n",
    );
}
