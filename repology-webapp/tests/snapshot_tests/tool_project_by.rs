// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use insta::assert_snapshot;
use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_construct_empty(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_construct_filled(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&noautoresolve=1").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_construct_ignores_invalid_fields(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=invalid&name_type=invalid&target_page=invalid").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_repository_not_specified(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?name_type=srcname&target_page=project_versions&name=shells/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_repository_not_found(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=invalid&name_type=srcname&target_page=project_versions&name=shells/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_name_type_not_specified(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&target_page=project_versions&name=shells/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_name_type_invalid(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=invalid&target_page=project_versions&name=shells/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_target_page_not_specified(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&name=shells/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_target_page_invalid(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=invalid&name=shells/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_package_name_not_found(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&name=invalid/invalid")
        .perform()
        .await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_ambiguity_with_disabled_autoresolve_html(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=project_versions&name=iperf&noautoresolve=1")
        .perform()
        .await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_failure_ambiguity_with_disabled_autoresolve_json(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=api_v1_project&name=iperf&noautoresolve=1")
        .perform()
        .await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_project_versions(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&name=shells/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_project_packages(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_packages&name=shells/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_project_information(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_information&name=shells/zsh")
        .perform()
        .await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_project_history(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_history&name=shells/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_project_badges(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_badges&name=shells/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_project_reports(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_reports&name=shells/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_badge_vertical_allrepos(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_vertical_allrepos&name=shells/zsh")
        .perform()
        .await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_badge_tiny_repos(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_tiny_repos&name=shells/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_badge_latest_versions(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_latest_versions&name=shells/zsh")
        .perform()
        .await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_badge_version_for_repo(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_version_for_repo&name=shells/zsh")
        .perform()
        .await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_badge_version_for_repo_custom_title(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_version_for_repo&name=shells/zsh&header=foo")
        .perform()
        .await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_target_api_v1_project(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=freebsd&name_type=srcname&target_page=api_v1_project&name=shells/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_ambiguous_with_autoresolve_html(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=project_versions&name=iperf").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_by_data"))]
async fn test_perform_success_ambiguous_with_autoresolve_json(pool: PgPool) {
    let response = Request::new(pool, "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=api_v1_project&name=iperf").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
