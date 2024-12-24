// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages", "log_data")
)]
async fn test_nonexistent(pool: PgPool) {
    check_response!(
        pool,
        "/log/10",
        status NOT_FOUND
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages", "log_data")
)]
async fn test_invalid_id(pool: PgPool) {
    check_response!(
        pool,
        "/log/foo",
        status BAD_REQUEST
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages", "log_data")
)]
async fn test_ongoing(pool: PgPool) {
    check_response!(
        pool,
        "/log/1",
        status OK,
        content_type TEXT_HTML,
        html_ok "allow_empty_tags,warnings_fatal",
        contains "ongoing"
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "common_packages", "log_data")
)]
async fn test_finished(pool: PgPool) {
    check_response!(
        pool,
        "/log/2",
        status OK,
        content_type TEXT_HTML,
        html_ok "allow_empty_tags,warnings_fatal",
        contains "successful"
    );
}
