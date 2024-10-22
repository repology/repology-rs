// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]
#![coverage(off)]

use sqlx::PgPool;

use repology_webapp_test_utils::check_response;

#[sqlx::test(migrator = "repology_common::MIGRATOR")]
async fn test_opensearch(pool: PgPool) {
    check_response!(
        pool,
        "/opensearch/maintainer.xml",
        status OK,
        content_type "application/xml",
        contains "={searchTerms}",
        xpath "string(/*[local-name()='OpenSearchDescription']/*[local-name()='ShortName'])" "Repology maintainers"
    );

    check_response!(
        pool,
        "/opensearch/project.xml",
        status OK,
        content_type "application/xml",
        contains "={searchTerms}",
        xpath "string(/*[local-name()='OpenSearchDescription']/*[local-name()='ShortName'])" "Repology projects"
    );
}
