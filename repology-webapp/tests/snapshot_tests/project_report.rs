// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::NaiveDate;
use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp::config::StaffAfkPeriod;
use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/project/nonexistent/report").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_orphaned_without_reports(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned-without-reports/report").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_orphaned_with_reports(pool: PgPool) {
    let response = Request::new(pool, "/project/orphaned-with-reports/report").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_too_many_reports(pool: PgPool) {
    let response = Request::new(pool, "/project/many-reports/report").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_xss_attempt(pool: PgPool) {
    let response = Request::new(pool, "/project/xss-attempt/report").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_all_flags(pool: PgPool) {
    let response = Request::new(pool, "/project/all-flags/report").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "project_report_data"))]
async fn test_afk(pool: PgPool) {
    let response = Request::new(pool, "/project/zsh/report")
        .with_staff_afk_period(StaffAfkPeriod {
            from: NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(),
            to: NaiveDate::from_ymd_opt(3000, 1, 1).unwrap(),
        })
        .perform()
        .await;

    assert_snapshot!(response.as_snapshot().unwrap());
}
