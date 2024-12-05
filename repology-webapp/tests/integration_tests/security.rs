// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("project_cves_data"))]
async fn test_recent_cves(pool: PgPool) {
    check_response!(
        pool,
        "/security/recent-cves",
        status OK,
        content_type TEXT_HTML,
        html_ok "allow_empty_tags,warnings_fatal",
        // we cannot really check presence of cve here because it depends on current date
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("project_cves_data"))]
async fn test_recent_cpes(pool: PgPool) {
    check_response!(
        pool,
        "/security/recent-cpes",
        status OK,
        content_type TEXT_HTML,
        html_ok "allow_empty_tags,warnings_fatal",
        contains "orphaned-with-cves",
        contains "foo",
        contains "manyranges",
        contains "bar",
        contains "tworanges",
        contains "baz",
    );
}
