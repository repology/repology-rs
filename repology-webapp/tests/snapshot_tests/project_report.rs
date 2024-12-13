// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_nonexistent(pool: PgPool) {
    uri_snapshot_test(pool, "/project/nonexistent/report").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_orphaned_without_reports(pool: PgPool) {
    uri_snapshot_test(pool, "/project/orphaned-without-reports/report").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_orphaned_with_reports(pool: PgPool) {
    uri_snapshot_test(pool, "/project/orphaned-with-reports/report").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_too_many_reports(pool: PgPool) {
    uri_snapshot_test(pool, "/project/many-reports/report").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_xss_attempt(pool: PgPool) {
    uri_snapshot_test(pool, "/project/xss-attempt/report").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_all_flags(pool: PgPool) {
    uri_snapshot_test(pool, "/project/all-flags/report").await;
}
