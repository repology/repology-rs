// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories")
)]
async fn test_construct_empty(pool: PgPool) {
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
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories")
)]
async fn test_construct_filled(pool: PgPool) {
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
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories")
)]
async fn test_construct_ignores_invalid_params(pool: PgPool) {
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
async fn test_perform_failure_repository_not_specified(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?name_type=srcname&target_page=project_versions&name=shells/zsh",
        status BAD_REQUEST,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        matches "repository.*was not specified"
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_failure_repository_not_found(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=invalid&name_type=srcname&target_page=project_versions&name=shells/zsh",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "repository was removed"
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_failure_name_type_not_specified(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&target_page=project_versions&name=shells/zsh",
        status BAD_REQUEST,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        matches "name type.*was not specified"
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_failure_name_type_invalid(pool: PgPool) {
    // name type invalid
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=invalid&target_page=project_versions&name=shells/zsh",
        status BAD_REQUEST,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        matches "name type.*invalid"
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_failure_target_page_not_specified(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&name=shells/zsh",
        status BAD_REQUEST,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        matches "target page.*was not specified"
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_failure_target_page_invalid(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=invalid&name=shells/zsh",
        status BAD_REQUEST,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        matches "target page.*is invalid"
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_failure_package_name_not_found(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&name=invalid/invalid",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        matches "package name.*not found"
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_failure_ambiguity_with_disabled_autoresolve_html(pool: PgPool) {
    // ambiguous without autoresolve
    check_response!(
        pool,
        "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=project_versions&name=iperf&noautoresolve=1",
        status MULTIPLE_CHOICES,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_failure_ambiguity_with_disabled_autoresolve_json(pool: PgPool) {
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
async fn test_perform_success_target_project_versions(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&name=shells/zsh",
        status FOUND,
        header_value "location" "/project/zsh/versions",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_success_target_project_packages(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_packages&name=shells/zsh",
        status FOUND,
        header_value "location" "/project/zsh/packages",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_success_target_project_information(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_information&name=shells/zsh",
        status FOUND,
        header_value "location" "/project/zsh/information",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_success_target_project_history(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_history&name=shells/zsh",
        status FOUND,
        header_value "location" "/project/zsh/history",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_success_target_project_badges(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_badges&name=shells/zsh",
        status FOUND,
        header_value "location" "/project/zsh/badges",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_success_target_project_reports(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_reports&name=shells/zsh",
        status FOUND,
        header_value "location" "/project/zsh/report",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_success_target_badge_vertical_allrepos(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_vertical_allrepos&name=shells/zsh",
        status FOUND,
        header_value "location" "/badge/vertical-allrepos/zsh.svg",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_success_target_badge_tiny_repos(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_tiny_repos&name=shells/zsh",
        status FOUND,
        header_value "location" "/badge/tiny-repos/zsh.svg",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_success_target_badge_latest_versions(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_latest_versions&name=shells/zsh",
        status FOUND,
        header_value "location" "/badge/latest-versions/zsh.svg",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_success_target_badge_version_for_repo(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_version_for_repo&name=shells/zsh",
        status FOUND,
        header_value "location" "/badge/version-for-repo/freebsd/zsh.svg",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_success_target_badge_version_for_repo_custom_title(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_version_for_repo&name=shells/zsh&header=foo",
        status FOUND,
        header_value "location" "/badge/version-for-repo/freebsd/zsh.svg?header=foo",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_success_target_api_v1_project(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=api_v1_project&name=shells/zsh",
        status FOUND,
        header_value "location" "/api/v1/project/zsh",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_success_ambiguous_with_autoresolve_html(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=project_versions&name=iperf",
        status FOUND,
        header_value "location" "/project/iperf2/versions", // assumes sorting
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_perform_success_ambiguous_with_autoresolve_json(pool: PgPool) {
    check_response!(
        pool,
        "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=api_v1_project&name=iperf",
        status FOUND,
        header_value "location" "/api/v1/project/iperf2", // assumes sorting
    );
}
