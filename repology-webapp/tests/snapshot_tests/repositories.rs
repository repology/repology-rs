// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics(pool: PgPool) {
    uri_snapshot_test(pool, "/repositories/statistics").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_newest(pool: PgPool) {
    uri_snapshot_test(pool, "/repositories/statistics/newest").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_pnewest(pool: PgPool) {
    uri_snapshot_test(pool, "/repositories/statistics/pnewest").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_outdated(pool: PgPool) {
    uri_snapshot_test(pool, "/repositories/statistics/outdated").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_poutdated(pool: PgPool) {
    uri_snapshot_test(pool, "/repositories/statistics/poutdated").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_total(pool: PgPool) {
    uri_snapshot_test(pool, "/repositories/statistics/total").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_nonunique(pool: PgPool) {
    uri_snapshot_test(pool, "/repositories/statistics/nonunique").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_vulnerable(pool: PgPool) {
    uri_snapshot_test(pool, "/repositories/statistics/vulnerable").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_statistics_sorted_pvulnerable(pool: PgPool) {
    uri_snapshot_test(pool, "/repositories/statistics/pvulnerable").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_packages(pool: PgPool) {
    uri_snapshot_test(pool, "/repositories/packages").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_graphs(pool: PgPool) {
    uri_snapshot_test(pool, "/repositories/graphs").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_updates(pool: PgPool) {
    // TODO: add some more fixure data, otherwise we're just testing an empty list
    uri_snapshot_test(pool, "/repositories/updates").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_repositories_fields(pool: PgPool) {
    // TODO: add some more fixure data, otherwise we're just testing an empty list
    uri_snapshot_test(pool, "/repositories/fields").await;
}
