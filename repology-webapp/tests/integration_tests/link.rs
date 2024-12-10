// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_nonexistent(pool: PgPool) {
    check_response!(
        pool,
        "/link/https://example.com/nonexistent",
        status NOT_FOUND
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_not_checked(pool: PgPool) {
    check_response!(
        pool,
        "/link/https://example.com/not-checked",
        status OK,
        contains "Not yet checked",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv4_failure(pool: PgPool) {
    check_response!(
        pool,
        "/link/https://example.com/ipv4-failure",
        status OK,
        contains "HTTP 404",
        contains "IPv6 was not checked",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv4_success(pool: PgPool) {
    check_response!(
        pool,
        "/link/https://example.com/ipv4-success",
        status OK,
        contains "OK (200)",
        contains "IPv6 was not checked",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv4_redirect(pool: PgPool) {
    check_response!(
        pool,
        "/link/https://example.com/ipv4-redirect",
        status OK,
        contains "permanent redirect",
        contains "https://example.com/ipv4-redirect-target",
        contains "IPv6 was not checked",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv6_failure(pool: PgPool) {
    check_response!(
        pool,
        "/link/https://example.com/ipv6-failure",
        status OK,
        contains "HTTP 404",
        contains "IPv4 was not checked",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv6_success(pool: PgPool) {
    check_response!(
        pool,
        "/link/https://example.com/ipv6-success",
        status OK,
        contains "OK (200)",
        contains "IPv4 check was skipped",
    );
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv6_redirect(pool: PgPool) {
    check_response!(
        pool,
        "/link/https://example.com/ipv6-redirect",
        status OK,
        contains "permanent redirect",
        contains "https://example.com/ipv6-redirect-target",
        contains "IPv4 check was skipped",
    );
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ssl_failure(pool: PgPool) {
    check_response!(
        pool,
        "/link/https://example.com/ssl-failure",
        status OK,
        contains "SSL error",
        contains "https://www.ssllabs.com/ssltest/analyze.html?d=example.com",
    );
}
