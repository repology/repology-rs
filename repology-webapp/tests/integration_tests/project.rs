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
        contains_not "binutils",
        contains_not "∗"
    );
    check_response!(
        pool,
        "/project/gcc/related",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "binutils",
        contains "/project/binutils/versions",
        contains "/project/binutils/related",
        contains "∗",
    );
    check_response!(
        pool,
        "/project/binutils/related",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "gcc",
        contains "/project/gcc/versions",
        contains "/project/gcc/related",
        contains "∗",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_history_data")
)]
async fn test_project_history(pool: PgPool) {
    check_response!(
        pool,
        "/project/nonexistent/history",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
    );
    check_response!(
        pool,
        "/project/orphaned-without-history/history",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
    );
    check_response!(
        pool,
        "/project/orphaned-with-history/history",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
    );

    check_response!(
        pool,
        "/project/zsh/history",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        // rather complex templates which may lead to whitespace before comma in some lists
        contains_not " ,",
        contains_not "\t,",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_project_cves(pool: PgPool) {
    check_response!(
        pool,
        "/project/cves/history",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
    );
    check_response!(
        pool,
        "/project/orphaned-without-cves/cves",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
    );
    check_response!(
        pool,
        "/project/orphaned-with-cves/cves",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "CVE-1-1",
    );

    check_response!(
        pool,
        "/project/manyranges/cves",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "CVE-2-2",
        contains "(-∞, +∞)",
        contains "(1.1, +∞)",
        contains "[1.2, +∞)",
        contains "(1.3, 1.4]",
        contains "[1.5, 1.6)",
        contains "(-∞, 1.7)",
        contains "(-∞, 1.8]",
    );

    check_response!(
        pool,
        "/project/tworanges/cves",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "CVE-3-3",
        contains "(1.3, 1.4]",
        contains "[1.5, 1.6)",
        // css class used for highlighted entries, here we don't expect any
        contains_not "version-outdated"
    );
    check_response!(
        pool,
        "/project/tworanges/cves?version=1.3",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains r#"<span class="version version-rolling">(1.3, 1.4]</span>"#,
    );
    check_response!(
        pool,
        "/project/tworanges/cves?version=1.4",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains r#"<span class="version version-outdated">(1.3, 1.4]</span>"#,
    );
}
