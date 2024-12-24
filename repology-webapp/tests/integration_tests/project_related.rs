// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "related_data")
)]
async fn test_nonexistent(pool: PgPool) {
    check_response!(
        pool,
        "/project/nonexistent/related",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Unknown project"
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "related_data")
)]
async fn test_orphaned(pool: PgPool) {
    check_response!(
        pool,
        "/project/orphaned/related",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Gone project"
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "related_data")
)]
async fn test_no_relations(pool: PgPool) {
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
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "related_data")
)]
async fn test_has_relations_a(pool: PgPool) {
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
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "related_data")
)]
async fn test_has_relations_b(pool: PgPool) {
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
