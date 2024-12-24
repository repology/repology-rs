// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_version_only_for_repo(pool: PgPool) {
    check_response!(pool, "/badge/version-only-for-repo/foo/bar.svg", status MOVED_PERMANENTLY, header_value "location" "/badge/version-for-repo/foo/bar.svg");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_version_only_for_repo_with_title(pool: PgPool) {
    check_response!(pool, "/badge/version-only-for-repo/foo/bar.svg?header=baz", status MOVED_PERMANENTLY, header_value "location" "/badge/version-for-repo/foo/bar.svg?header=baz");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_project_root(pool: PgPool) {
    check_response!(pool, "/project/zsh", status MOVED_PERMANENTLY, header_value "location" "/project/zsh/versions");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_metapackage(pool: PgPool) {
    check_response!(pool, "/metapackage/zsh", status MOVED_PERMANENTLY, header_value "location" "/project/zsh/versions");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_metapackage_versions(pool: PgPool) {
    check_response!(pool, "/metapackage/zsh/versions", status MOVED_PERMANENTLY, header_value "location" "/project/zsh/versions");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_metapackage_packages(pool: PgPool) {
    check_response!(pool, "/metapackage/zsh/packages", status MOVED_PERMANENTLY, header_value "location" "/project/zsh/packages");
}
