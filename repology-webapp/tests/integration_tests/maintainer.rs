// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_nonexistent(pool: PgPool) {
    check_response!(
        pool,
        "/maintainer/nonexistent@example.com",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Unknown maintainer",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_orphaned(pool: PgPool) {
    check_response!(
        pool,
        "/maintainer/orphaned@example.com",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Gone maintainer",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_orphaned_in_future(pool: PgPool) {
    check_response!(
        pool,
        "/maintainer/orphaned-in-future@example.com",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "Gone maintainer",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_active(pool: PgPool) {
    check_response!(
        pool,
        "/maintainer/active@example.com",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        // fallback section
        contains_not "fallback maintainer",
        // contact section
        contains "mailto:active@example.com",
        // repositories section
        contains "FreeBSD",
        // categories section
        contains "games",
        // not testing similar maintainers for now
        // not testing projects list for now
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_fallback(pool: PgPool) {
    check_response!(
        pool,
        "/maintainer/fallback-mnt-foo@repology",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        // fallback section
        contains "fallback maintainer",
        // contact section
        contains_not "mailto:active@example.com",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "maintainer_data"))]
async fn test_no_vuln_column(pool: PgPool) {
    // Maintainer not updated for a long time, without vulnerable projects
    // counter filled.
    check_response!(
        pool,
        "/maintainer/no-vuln-column@example.com",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        // enough to just be deserialized without errors
    );
}
