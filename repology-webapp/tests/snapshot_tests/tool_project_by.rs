// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories")
)]
async fn test_project_by_construct(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/tools/project-by").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&noautoresolve=1").await;
    uri_snapshot_test(
        pool.clone(),
        "/tools/project-by?repo=invalid&name_type=invalid&target_page=invalid",
    )
    .await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_project_by_failures(pool: PgPool) {
    uri_snapshot_test(
        pool.clone(),
        "/tools/project-by?name_type=srcname&target_page=project_versions&name=shells/zsh",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=invalid&name_type=srcname&target_page=project_versions&name=shells/zsh").await;
    uri_snapshot_test(
        pool.clone(),
        "/tools/project-by?repo=freebsd&target_page=project_versions&name=shells/zsh",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=invalid&target_page=project_versions&name=shells/zsh").await;
    uri_snapshot_test(
        pool.clone(),
        "/tools/project-by?repo=freebsd&name_type=srcname&name=shells/zsh",
    )
    .await;
    uri_snapshot_test(
        pool.clone(),
        "/tools/project-by?repo=freebsd&name_type=srcname&target_page=invalid&name=shells/zsh",
    )
    .await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&name=invalid/invalid").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=project_versions&name=iperf&noautoresolve=1").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=api_v1_project&name=iperf&noautoresolve=1").await;
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_by_data")
)]
async fn test_project_by_ok(pool: PgPool) {
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_versions&name=shells/zsh").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_packages&name=shells/zsh").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_information&name=shells/zsh").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_history&name=shells/zsh").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_badges&name=shells/zsh").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=project_reports&name=shells/zsh").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_vertical_allrepos&name=shells/zsh").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_tiny_repos&name=shells/zsh").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_latest_versions&name=shells/zsh").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_version_for_repo&name=shells/zsh").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=badge_version_for_repo&name=shells/zsh&header=foo").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=freebsd&name_type=srcname&target_page=api_v1_project&name=shells/zsh").await;
    uri_snapshot_test(pool.clone(), "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=project_versions&name=iperf").await;
    uri_snapshot_test(
        pool.clone(),
        "/tools/project-by?repo=ubuntu_24&name_type=srcname&target_page=api_v1_project&name=iperf",
    )
    .await;
}
