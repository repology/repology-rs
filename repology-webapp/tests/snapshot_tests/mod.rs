// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod api_v1_project;
mod api_v1_projects;
mod badge_latest_versions;
mod badge_repository_big;
mod badge_tiny_repos;
mod badge_version_for_repo;
mod badge_vertical_allrepos;
mod feed_maintainer_atom;
mod feed_maintainer_html;
mod feed_repository_atom;
mod feed_repository_html;
mod graphs_map_repo_size_fresh;
mod graphs_repository;
mod graphs_total;
mod index;
mod legacy_redirects;
mod log;
mod maintainer;
mod maintainers;
mod opensearch;
mod problems;
mod project_badges;
mod project_cves;
mod project_history;
mod project_information;
mod project_packages;
mod project_related;
mod project_report;
mod project_versions;
mod projects;
mod repositories;
mod repository;
mod security;
mod tool_project_by;
mod trivial_pages;

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[track_caller]
async fn uri_snapshot_test(pool: PgPool, uri: &str) {
    insta::assert_snapshot!(uri, Request::new(pool, uri).perform().await.as_snapshot().unwrap());
}
