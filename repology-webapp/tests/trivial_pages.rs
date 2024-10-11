// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_html;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_trivial_pages(pool: PgPool) {
    check_html!(pool, "/api", "Terms of use");
    check_html!(pool, "/api/v1", "Terms of use");
    check_html!(pool, "/docs/about", "About");
    check_html!(pool, "/docs/bots", "+https://repology.org/docs/bots");
    check_html!(pool, "/docs", "Documentation");
    check_html!(pool, "/docs/not_supported", "Hyperbola");
    check_html!(pool, "/docs/requirements", "Rational");
    check_html!(pool, "/news", "Added");
    check_html!(pool, "/tools", "Project by package name");
}
