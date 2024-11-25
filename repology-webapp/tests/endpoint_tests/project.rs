// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

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

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages")
)]
async fn test_project_badges(pool: PgPool) {
    check_response!(
        pool,
        "/project/nonexistent/badges",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Unknown project"
    );
    check_response!(
        pool,
        "/project/orphaned/badges",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Gone project"
    );

    check_response!(
        pool,
        "/project/zsh/badges",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Badges for <strong>zsh",
    );

    // XXX: more tests
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "related_data")
)]
async fn test_project_related(pool: PgPool) {
    check_response!(
        pool,
        "/project/nonexistent/related",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Unknown project"
    );
    check_response!(
        pool,
        "/project/orphaned/related",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Gone project"
    );

    check_response!(
        pool,
        "/project/zsh/related",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains_not "gcc",
        contains_not "binutils"
    );
    check_response!(
        pool,
        "/project/gcc/related",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "binutils"
    );
    check_response!(
        pool,
        "/project/binutils/related",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "gcc"
    );
}