// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_project_versions(pool: PgPool) {
    check_response!(
        pool,
        "/project/nonexistent/versions",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Unknown project"
    );
    check_response!(
        pool,
        "/project/orphaned/versions",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Gone project"
    );

    check_response!(
        pool,
        "/project/zsh/versions",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Versions for <strong>zsh",
        contains "FreeBSD",
        contains "1.1"
    );

    // XSS test
    check_response!(
        pool,
        "/project/%3Cmarquee%3E%26nbsp%3B/versions",
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains_not "<marquee>",
        contains_not "&nbsp;"
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_project_packages(pool: PgPool) {
    check_response!(
        pool,
        "/project/nonexistent/packages",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Unknown project"
    );
    check_response!(
        pool,
        "/project/orphaned/packages",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Gone project"
    );

    check_response!(
        pool,
        "/project/zsh/packages",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Packages for <strong>zsh",
        contains "FreeBSD",
        contains "1.1"
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_project_information(pool: PgPool) {
    check_response!(
        pool,
        "/project/nonexistent/information",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Unknown project"
    );
    check_response!(
        pool,
        "/project/orphaned/information",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Gone project"
    );

    check_response!(
        pool,
        "/project/zsh/information",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Information for <strong>zsh",
    );

    // XXX: more tests
}
