// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_legacy_redirects(pool: PgPool) {
    check_response!(pool, "/badge/version-only-for-repo/foo/bar.svg", status MOVED_PERMANENTLY, header_value "location" "/badge/version-for-repo/foo/bar.svg");
    check_response!(pool, "/badge/version-only-for-repo/foo/bar.svg?header=baz", status MOVED_PERMANENTLY, header_value "location" "/badge/version-for-repo/foo/bar.svg?header=baz");

    check_response!(pool, "/metapackage/zsh", status MOVED_PERMANENTLY, header_value "location" "/project/zsh/versions");
    check_response!(pool, "/metapackage/zsh/versions", status MOVED_PERMANENTLY, header_value "location" "/project/zsh/versions");
}
