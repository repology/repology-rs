// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_nonexistent(pool: PgPool) {
    check_response!(
        pool,
        "/repository/nonexistent",
        status NOT_FOUND,
        // currently no html page for 404
        //content_type "text/html",
        //html_ok "allow_empty_tags,warnings_fatal",
        //contains "Unknown repositry",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_orphaned(pool: PgPool) {
    check_response!(
        pool,
        "/repository/orphaned",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Gone repository",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_empty(pool: PgPool) {
    check_response!(
        pool,
        "/repository/empty",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains_not "Gone repository",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_stripped(pool: PgPool) {
    // test handling minimal data in database: empty metadata, and
    // no used_package_link_types; in prod this case is possible
    // for repositories removed long time ago
    check_response!(
        pool,
        "/repository/stripped",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Stripped",
        contains "homepage or download links",
        contains "package recipes or sources",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("repository_data"))]
async fn test_normal(pool: PgPool) {
    check_response!(
        pool,
        "/repository/good",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Good",
        contains_not "homepage or download links",
        contains_not "package recipes or sources",
        contains "https://example.com/goodrepo",
    );
}
