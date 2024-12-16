// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_api(pool: PgPool) {
    check_response!(pool, "/api", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Terms of use");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_api_v1(pool: PgPool) {
    check_response!(pool, "/api/v1", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Terms of use");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_about(pool: PgPool) {
    check_response!(pool, "/docs/about", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "About");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_bots(pool: PgPool) {
    check_response!(pool, "/docs/bots", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "+https://repology.org/docs/bots");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs(pool: PgPool) {
    check_response!(pool, "/docs", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Documentation");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_not_supported(pool: PgPool) {
    check_response!(pool, "/docs/not_supported", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Hyperbola");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_docs_requirements(pool: PgPool) {
    check_response!(pool, "/docs/requirements", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Rational");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_news(pool: PgPool) {
    check_response!(pool, "/news", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Added");
}

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_tools(pool: PgPool) {
    check_response!(pool, "/tools", status OK, content_type "text/html", html_ok "allow_empty_tags,warnings_fatal", contains "Project by package name");
}
