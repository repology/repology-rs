// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories")
)]
async fn test_project_by_construct(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",

        contains_not r#"<option value="freebsd" selected>"#,
        contains_not r#"<option value="srcname" selected>"#,
        contains_not r#"<option value="project_versions" selected>"#,
        contains_not r#"<input type="checkbox" name="noautoresolve" checked>"#,

        contains_not r#"/tools/project-by?"#,
    );

    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&noautoresolve=1",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",

        contains r#"<option value="freebsd" selected>"#,
        contains r#"<option value="srcname" selected>"#,
        contains r#"<option value="project_versions" selected>"#,
        contains r#"<input type="checkbox" name="noautoresolve" checked>"#,

        contains "/tools/project-by?repo=freebsd&amp;name_type=srcname&amp;target_page=project_versions&amp;noautoresolve=1&amp;name=&lt;NAME&gt;",
    );

    // still works when passed invalid params, the same way as if these were not specified
    check_response!(
        pool,
        "/tools/project-by?repo=invalid&name_type=invalid&target_page=invalid",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",

        contains_not r#"<option value="freebsd" selected>"#,
        contains_not r#"<option value="srcname" selected>"#,
        contains_not r#"<option value="project_versions" selected>"#,
        contains_not r#"<input type="checkbox" name="noautoresolve" checked>"#,

        contains_not "/tools/project-by?",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_project_by_failures(pool: PgPool) {
    // repository not specified
    check_response!(
        pool,
        "/tools/project-by?name_type=srcname&target_page=project_versions&name=shells/zsh",
        status BAD_REQUEST,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        matches "repository.*was not specified"
    );

    // repository invalid
    check_response!(
        pool,
        "/tools/project-by?repo=invalid&name_type=srcname&target_page=project_versions&name=shells/zsh",
        status BAD_REQUEST,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "repository was removed"
    );

    // name type not specified
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&target_page=project_versions&name=shells/zsh",
        status BAD_REQUEST,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        matches "name type.*was not specified"
    );

    // name type invalid
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=invalid&target_page=project_versions&name=shells/zsh",
        status BAD_REQUEST,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        matches "name type.*invalid"
    );

    // target page not specified
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&name=shells/zsh",
        status BAD_REQUEST,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        matches "target page.*was not specified"
    );

    // target page not speified
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=invalid&name=shells/zsh",
        status BAD_REQUEST,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        matches "target page.*is invalid"
    );

    // not found
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&name=invalid/invalid",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        matches "package name.*not found"
    );

    // ambiguous without autoresolve
    check_response!(
        pool,
        "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=project_versions&name=iperf&noautoresolve=1",
        status MULTIPLE_CHOICES,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
    );
    check_response!(
        pool,
        "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=api_v1_project&name=iperf&noautoresolve=1",
        status MULTIPLE_CHOICES,
        content_type "application/json",
        json r#"
        {
            "_comment": "Ambiguous redirect, multiple target projects are possible",
            "targets": {
                "iperf2": "/api/v1/project/iperf2",
                "iperf3": "/api/v1/project/iperf3"
            }
        }
        "#
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_project_by_ok(pool: PgPool) {
    // all target page types
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&name=shells/zsh",
        status FOUND,
        header_value "location" "/project/zsh/versions",
    );
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_packages&name=shells/zsh",
        status FOUND,
        header_value "location" "/project/zsh/packages",
    );
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_information&name=shells/zsh",
        status FOUND,
        header_value "location" "/project/zsh/information",
    );
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_history&name=shells/zsh",
        status FOUND,
        header_value "location" "/project/zsh/history",
    );
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_badges&name=shells/zsh",
        status FOUND,
        header_value "location" "/project/zsh/badges",
    );
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_reports&name=shells/zsh",
        status FOUND,
        header_value "location" "/project/zsh/report",
    );

    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_vertical_allrepos&name=shells/zsh",
        status FOUND,
        header_value "location" "/badge/vertical-allrepos/zsh.svg",
    );
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_tiny_repos&name=shells/zsh",
        status FOUND,
        header_value "location" "/badge/tiny-repos/zsh.svg",
    );
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_latest_versions&name=shells/zsh",
        status FOUND,
        header_value "location" "/badge/latest-versions/zsh.svg",
    );
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_version_for_repo&name=shells/zsh",
        status FOUND,
        header_value "location" "/badge/version-for-repo/freebsd/zsh.svg",
    );
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_version_for_repo&name=shells/zsh&header=foo",
        status FOUND,
        header_value "location" "/badge/version-for-repo/freebsd/zsh.svg?header=foo",
    );

    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=api_v1_project&name=shells/zsh",
        status FOUND,
        header_value "location" "/api/v1/project/zsh",
    );

    // ambiguous with autoresolve
    check_response!(
        pool,
        "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=project_versions&name=iperf",
        status FOUND,
        header_value "location" "/project/iperf2/versions", // assumes sorting
    );
    check_response!(
        pool,
        "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=api_v1_project&name=iperf",
        status FOUND,
        header_value "location" "/api/v1/project/iperf2", // assumes sorting
    );
}
