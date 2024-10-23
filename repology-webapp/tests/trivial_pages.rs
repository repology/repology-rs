// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_trivial_pages(pool: PgPool) {
    check_response!(pool, "/api", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Terms of use");
    check_response!(pool, "/api/v1", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Terms of use");
    check_response!(pool, "/docs/about", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "About");
    check_response!(pool, "/docs/bots", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "+https://repology.org/docs/bots");
    check_response!(pool, "/docs", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Documentation");
    check_response!(pool, "/docs/not_supported", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Hyperbola");
    check_response!(pool, "/docs/requirements", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Rational");
    check_response!(pool, "/news", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Added");
    check_response!(pool, "/tools", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Project by package name");
}
