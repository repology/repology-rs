// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod api_v1_project;
mod api_v1_projects;
mod badge_tiny_repos;
mod badges;
mod feeds;
mod graphs; // XXX: may produce false positives due to moving timestamps
mod graphs_map_repo_size_fresh;
mod legacy_redirects;
mod log;
mod maintainer;
mod opensearch;
mod problems;
mod project_badges;
mod project_cves;
mod project_history;
mod project_information;
mod project_packages;
mod project_related;
mod project_versions;
mod projects;
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
