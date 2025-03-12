// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use insta::assert_snapshot;
use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_missing_extension(pool: PgPool) {
    let response = Request::new(pool, "/badge/vertical-allrepos/zsh").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_base(pool: PgPool) {
    let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

mod test_header {
    use super::*;

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_custom(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?header=Packages").perform().await;
        assert_snapshot!(response.as_snapshot().unwrap());
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_custom_empty(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?header=").perform().await;
        assert_snapshot!(response.as_snapshot().unwrap());
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_no_packages(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/unpackaged.svg").perform().await;
        assert_snapshot!(response.as_snapshot().unwrap());
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_no_packages_custom(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/unpackaged.svg?header=Packages").perform().await;
        assert_snapshot!(response.as_snapshot().unwrap());
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_no_packages_custom_empty(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/unpackaged.svg?header=").perform().await;
        assert_snapshot!(response.as_snapshot().unwrap());
    }
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_allow_ignored(pool: PgPool) {
    let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?allow_ignored=1").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_minversion(pool: PgPool) {
    let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?minversion=1.0").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

mod test_repository_filters {
    use super::*;

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_exclude_unsupported(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?exclude_unsupported=1").perform().await;
        assert_snapshot!(response.as_snapshot().unwrap());
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_exclude_sources_site(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?exclude_sources=site").perform().await;
        assert_snapshot!(response.as_snapshot().unwrap());
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_exclude_sources_repositry(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository").perform().await;
        assert_snapshot!(response.as_snapshot().unwrap());
    }

    #[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
    async fn test_exclude_sources_many(pool: PgPool) {
        let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?exclude_sources=repository,site").perform().await;
        assert_snapshot!(response.as_snapshot().unwrap());
    }
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages"))]
async fn test_columns(pool: PgPool) {
    let response = Request::new(pool, "/badge/vertical-allrepos/zsh.svg?columns=4").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
