// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use super::uri_snapshot_test;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_missing_extension(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/vertical-allrepos/zsh").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_base(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/vertical-allrepos/zsh.svg").await;
}

mod test_header {
    use super::*;

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_custom(pool: PgPool) {
        uri_snapshot_test(pool, "/badge/vertical-allrepos/zsh.svg?header=Packages").await;
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_custom_empty(pool: PgPool) {
        uri_snapshot_test(pool, "/badge/vertical-allrepos/zsh.svg?header=").await;
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_no_packages(pool: PgPool) {
        uri_snapshot_test(pool, "/badge/vertical-allrepos/unpackaged.svg").await;
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_no_packages_custom(pool: PgPool) {
        uri_snapshot_test(pool, "/badge/vertical-allrepos/unpackaged.svg?header=Packages").await;
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_no_packages_custom_empty(pool: PgPool) {
        uri_snapshot_test(pool, "/badge/vertical-allrepos/unpackaged.svg?header=").await;
    }
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_allow_ignored(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/vertical-allrepos/zsh.svg?allow_ignored=1").await;
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_minversion(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/vertical-allrepos/zsh.svg?minversion=1.0").await;
}

mod test_repository_filters {
    use super::*;

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_exclude_unsupported(pool: PgPool) {
        uri_snapshot_test(pool, "/badge/vertical-allrepos/zsh.svg?exclude_unsupported=1").await;
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_exclude_sources_site(pool: PgPool) {
        uri_snapshot_test(pool, "/badge/vertical-allrepos/zsh.svg?exclude_sources=site").await;
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_exclude_sources_repositry(pool: PgPool) {
        uri_snapshot_test(pool, "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository").await;
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_exclude_sources_many(pool: PgPool) {
        uri_snapshot_test(pool, "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository,site").await;
    }
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_columns(pool: PgPool) {
    uri_snapshot_test(pool, "/badge/vertical-allrepos/zsh.svg?columns=4").await;
}
