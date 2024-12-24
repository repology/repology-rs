// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_history_data")
)]
async fn test_nonexistent(pool: PgPool) {
    check_response!(
        pool,
        "/project/nonexistent/history",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_history_data")
)]
async fn test_orphaned_without_history(pool: PgPool) {
    check_response!(
        pool,
        "/project/orphaned-without-history/history",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_history_data")
)]
async fn test_orphaned_with_history(pool: PgPool) {
    check_response!(
        pool,
        "/project/orphaned-with-history/history",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_history_data")
)]
async fn test_normal(pool: PgPool) {
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
