// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "common_maintainers", "maintainer_feed_data"))]
async fn test_nonexistent_maintainer(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/nonexistent@example.com/feed-for-repo/freebsd").perform().await;
    // we don't currently check for maintainer existence, so it's just an empty feed
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "common_maintainers", "maintainer_feed_data"))]
async fn test_nonexistent_repository(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/johndoe@example.com/feed-for-repo/nonexistent").perform().await;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "common_packages", "common_maintainers", "maintainer_feed_data"))]
async fn test_base(pool: PgPool) {
    let response = Request::new(pool, "/maintainer/johndoe@example.com/feed-for-repo/freebsd").perform().await;
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.header_value_str("content-type").unwrap(), Some("text/html"));
    assert!(response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));

    assert!(response.text().unwrap().contains(">111<"));
    assert!(response.text().unwrap().contains(">222<"));
    assert!(response.text().unwrap().contains(">333<"));
    assert!(response.text().unwrap().contains(">444<"));
    assert!(response.text().unwrap().contains(">555<"));
    assert!(response.text().unwrap().contains(">666<"));
}
