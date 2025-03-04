// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::NaiveDate;
use sqlx::PgPool;

use repology_webapp::config::StaffAfkPeriod;
use repology_webapp_test_utils::Request;

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

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_afk(pool: PgPool) {
    insta::assert_snapshot!(
        "/project/zsh/report",
        Request::new(pool, "/project/zsh/report")
            .with_staff_afk_period(StaffAfkPeriod {
                from: NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(),
                to: NaiveDate::from_ymd_opt(3000, 1, 1).unwrap(),
            })
            .perform()
            .await
            .as_snapshot()
            .unwrap()
    );
}
