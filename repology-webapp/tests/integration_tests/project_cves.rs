// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_nonexistent(pool: PgPool) {
    check_response!(
        pool,
        "/project/nonexistent/cves",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_orphaned_without_cves(pool: PgPool) {
    check_response!(
        pool,
        "/project/orphaned-without-cves/cves",
        status NOT_FOUND,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_orphaned_with_cves(pool: PgPool) {
    check_response!(
        pool,
        "/project/orphaned-with-cves/cves",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "CVE-1-1",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_ranges(pool: PgPool) {
    check_response!(
        pool,
        "/project/manyranges/cves",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "CVE-2-2",
        contains "(-∞, +∞)",
        contains "(1.1, +∞)",
        contains "[1.2, +∞)",
        contains "(1.3, 1.4]",
        contains "[1.5, 1.6)",
        contains "(-∞, 1.7)",
        contains "(-∞, 1.8]",
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_version_not_highlighted(pool: PgPool) {
    check_response!(
        pool,
        "/project/tworanges/cves",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains "CVE-3-3",
        contains "(1.3, 1.4]",
        contains "[1.5, 1.6)",
        // css class used for highlighted entries, here we don't expect any
        contains_not "version-outdated"
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_open_range_version_not_highlighted(pool: PgPool) {
    check_response!(
        pool,
        "/project/tworanges/cves?version=1.3",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains r#"<span class="version version-rolling">(1.3, 1.4]</span>"#,
    );
}

#[sqlx::test(
    migrator = "repology_common::MIGRATOR",
    fixtures("common_repositories", "project_cves_data")
)]
async fn test_version_highlighted(pool: PgPool) {
    check_response!(
        pool,
        "/project/tworanges/cves?version=1.4",
        status OK,
        content_type "text/html",
        html_ok "allow_empty_tags,warnings_fatal",
        contains r#"<span class="version version-outdated">(1.3, 1.4]</span>"#,
    );
}
