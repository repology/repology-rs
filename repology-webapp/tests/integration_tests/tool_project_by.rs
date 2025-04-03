// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};
use serde_json::json;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_construct_empty(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));

    assert!(!response.text().unwrap().contains(r#"<option value="freebsd" selected>"#));
    assert!(!response.text().unwrap().contains(r#"<option value="srcname" selected>"#));
    assert!(!response.text().unwrap().contains(r#"<option value="project_versions" selected>"#));
    assert!(!response.text().unwrap().contains(r#"<input type="checkbox" name="noautoresolve" checked>"#));

    assert!(!response.text().unwrap().contains(r#"/tools/project-by?"#));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_construct_filled(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&noautoresolve=1").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));

    assert!(response.text().unwrap().contains(r#"<option value="freebsd" selected>"#));
    assert!(response.text().unwrap().contains(r#"<option value="srcname" selected>"#));
    assert!(response.text().unwrap().contains(r#"<option value="project_versions" selected>"#));
    assert!(response.text().unwrap().contains(r#"<input type="checkbox" name="noautoresolve" checked>"#));

    assert!(
        response
            .text()
            .unwrap()
            .contains("/tools/project-by?repo=freebsd&#38;name_type=srcname&#38;target_page=project_versions&#38;noautoresolve=1&#38;name=&#60;NAME&#62;")
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_construct_ignores_invalid_params(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=invalid&name_type=invalid&target_page=invalid").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));

    assert!(!response.text().unwrap().contains(r#"<option value="freebsd" selected>"#));
    assert!(!response.text().unwrap().contains(r#"<option value="srcname" selected>"#));
    assert!(!response.text().unwrap().contains(r#"<option value="project_versions" selected>"#));
    assert!(!response.text().unwrap().contains(r#"<input type="checkbox" name="noautoresolve" checked>"#));

    assert!(!response.text().unwrap().contains("/tools/project-by?"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_repository_not_specified(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?name_type=srcname&target_page=project_versions&name=shells/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(regex::Regex::new(r"repository.*was not specified").unwrap().is_match(response.text().unwrap()));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_repository_not_found(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=invalid&name_type=srcname&target_page=project_versions&name=shells/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(response.text().unwrap().contains("repository was removed"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_name_type_not_specified(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&target_page=project_versions&name=shells/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(regex::Regex::new(r"name type.*was not specified").unwrap().is_match(response.text().unwrap()));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_name_type_invalid(pool: PgPool) {
    // name type invalid
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=invalid&target_page=project_versions&name=shells/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(regex::Regex::new(r"name type.*invalid").unwrap().is_match(response.text().unwrap()));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_target_page_not_specified(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&name=shells/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(regex::Regex::new(r"target page.*was not specified").unwrap().is_match(response.text().unwrap()));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_target_page_invalid(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=invalid&name=shells/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(regex::Regex::new(r"target page.*is invalid").unwrap().is_match(response.text().unwrap()));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_package_name_not_found(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&name=invalid/invalid")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
    assert!(regex::Regex::new(r"package name.*not found").unwrap().is_match(response.text().unwrap()));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_ambiguity_with_disabled_autoresolve_html(pool: PgPool) {
    // ambiguous without autoresolve
    let response = Request::new(pool, "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=project_versions&name=iperf&noautoresolve=1")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::MULTIPLE_CHOICES);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_ambiguity_with_disabled_autoresolve_json(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=api_v1_project&name=iperf&noautoresolve=1")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::MULTIPLE_CHOICES);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/json"));
    pretty_assertions::assert_eq!(
        response.json().unwrap(),
        json!(
            {
                "_comment": "Ambiguous redirect, multiple target projects are possible",
                "targets": {
                    "iperf2": "/api/v1/project/iperf2",
                    "iperf3": "/api/v1/project/iperf3"
                }
            }
        )
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_project_versions(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&name=shells/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/project/zsh/versions"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_project_packages(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_packages&name=shells/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/project/zsh/packages"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_project_information(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_information&name=shells/zsh")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/project/zsh/information"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_project_history(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_history&name=shells/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/project/zsh/history"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_project_badges(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_badges&name=shells/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/project/zsh/badges"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_project_reports(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_reports&name=shells/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/project/zsh/report"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_badge_vertical_allrepos(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_vertical_allrepos&name=shells/zsh")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/badge/vertical-allrepos/zsh.svg"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_badge_tiny_repos(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_tiny_repos&name=shells/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/badge/tiny-repos/zsh.svg"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_badge_latest_versions(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_latest_versions&name=shells/zsh")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/badge/latest-versions/zsh.svg"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_badge_version_for_repo(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_version_for_repo&name=shells/zsh")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/badge/version-for-repo/freebsd/zsh.svg"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_badge_version_for_repo_custom_title(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_version_for_repo&name=shells/zsh&header=foo")
        .perform()
        .await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/badge/version-for-repo/freebsd/zsh.svg?header=foo"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_api_v1_project(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=api_v1_project&name=shells/zsh").perform().await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    assert_eq!(response.header_value_str("location").unwrap(), Some("/api/v1/project/zsh"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_ambiguous_with_autoresolve_html(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=project_versions&name=iperf").perform().await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    // assumes sorting when chosing target
    assert_eq!(response.header_value_str("location").unwrap(), Some("/project/iperf2/versions"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_ambiguous_with_autoresolve_json(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=api_v1_project&name=iperf").perform().await;
    assert_eq!(response.status(), http::StatusCode::FOUND);
    // assumes sorting when chosing target
    assert_eq!(response.header_value_str("location").unwrap(), Some("/api/v1/project/iperf2"));
}
