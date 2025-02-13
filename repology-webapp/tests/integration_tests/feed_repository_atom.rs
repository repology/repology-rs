// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "repository_feed_data"))]
async fn test_nonexistent_repository(pool: PgPool) {
    let response = Request::new(pool, "/repository/nonexistent/feed/atom").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "repository_feed_data"))]
async fn test_base(pool: PgPool) {
    let response = Request::new(pool, "/repository/freebsd/feed/atom").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("application/atom+xml"));

    assert_eq!(response.xpath("count(/*[local-name()='feed']/*[local-name()='entry'])").unwrap(), 6_f64);
    assert_eq!(
        response.xpath("string(/*[local-name()='feed']/*[local-name()='entry'][1]/*[local-name()='title'])").unwrap(),
        "zsh is no longer tracked"
    );
    assert_eq!(response.xpath("string(/*[local-name()='feed']/*[local-name()='entry'][2]/*[local-name()='title'])").unwrap(), "zsh 666 is ignored");
    assert_eq!(
        response.xpath("string(/*[local-name()='feed']/*[local-name()='entry'][3]/*[local-name()='title'])").unwrap(),
        "zsh 222 is outdated by 333, 444"
    );
    assert_eq!(response.xpath("string(/*[local-name()='feed']/*[local-name()='entry'][4]/*[local-name()='title'])").unwrap(), "zsh 555 is outdated");
    assert_eq!(
        response.xpath("string(/*[local-name()='feed']/*[local-name()='entry'][5]/*[local-name()='title'])").unwrap(),
        "zsh 111 is up to date"
    );
    assert_eq!(response.xpath("string(/*[local-name()='feed']/*[local-name()='entry'][6]/*[local-name()='title'])").unwrap(), "zsh is now tracked");
    assert!(!response.text().unwrap().contains("\n\n"));
}
