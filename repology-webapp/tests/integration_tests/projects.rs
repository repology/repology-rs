// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;

use repology_webapp_test_utils::{HtmlValidationFlags, Request};

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories"))]
async fn test_params_retained_by_the_form(pool: PgPool) {
    for url in ["/projects/", "/projects/foo/", "/projects/..foo/"] {
        let parametrized_url = url.to_string()
            + "?search=xsearchx&maintainer=xmaintainerx&category=xcategoryx&inrepo=freebsd&notinrepo=ubuntu_24&repos=970-971&families=972-973&repos_newest=974-975&families_newest=976-977&newest=1&outdated=1&problematic=1&vulnerable=1&has_related=1";
        let naked_response = Request::new(pool.clone(), url).perform().await;
        let param_response = Request::new(pool.clone(), &parametrized_url).perform().await;

        assert_eq!(naked_response.status(), http::StatusCode::OK);
        assert_eq!(param_response.status(), http::StatusCode::OK);
        assert_eq!(naked_response.header_value_str("content-type").unwrap(), Some("text/html"));
        assert_eq!(param_response.header_value_str("content-type").unwrap(), Some("text/html"));
        assert!(naked_response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));
        assert!(param_response.is_html_valid(HtmlValidationFlags::ALLOW_EMPTY_TAGS | HtmlValidationFlags::WARNINGS_ARE_FATAL));

        assert!(!naked_response.text().unwrap().contains("xsearchx"));
        assert!(param_response.text().unwrap().contains("xsearchx"));
        assert!(!naked_response.text().unwrap().contains("xmaintainerx"));
        assert!(param_response.text().unwrap().contains("xmaintainerx"));
        assert!(!naked_response.text().unwrap().contains("xcategoryx"));
        assert!(param_response.text().unwrap().contains("xcategoryx"));

        let re = regex::Regex::new(r"freebsd.*selected").unwrap();
        assert!(!naked_response.text().unwrap().lines().any(|line| re.is_match(line)));
        assert!(param_response.text().unwrap().lines().any(|line| re.is_match(line)));

        let re = regex::Regex::new(r"ubuntu_24.*selected").unwrap();
        assert!(!naked_response.text().unwrap().lines().any(|line| re.is_match(line)));
        assert!(param_response.text().unwrap().lines().any(|line| re.is_match(line)));

        assert!(!naked_response.text().unwrap().contains("970"));
        assert!(param_response.text().unwrap().contains("970"));
        assert!(!naked_response.text().unwrap().contains("971"));
        assert!(param_response.text().unwrap().contains("971"));
        assert!(!naked_response.text().unwrap().contains("972"));
        assert!(param_response.text().unwrap().contains("972"));
        assert!(!naked_response.text().unwrap().contains("973"));
        assert!(param_response.text().unwrap().contains("973"));
        assert!(!naked_response.text().unwrap().contains("974"));
        assert!(param_response.text().unwrap().contains("974"));
        assert!(!naked_response.text().unwrap().contains("975"));
        assert!(param_response.text().unwrap().contains("975"));
        assert!(!naked_response.text().unwrap().contains("976"));
        assert!(param_response.text().unwrap().contains("976"));
        assert!(!naked_response.text().unwrap().contains("977"));
        assert!(param_response.text().unwrap().contains("977"));

        let re = regex::Regex::new(r"newest.*checked").unwrap();
        assert!(!naked_response.text().unwrap().lines().any(|line| re.is_match(line)));
        assert!(param_response.text().unwrap().lines().any(|line| re.is_match(line)));

        let re = regex::Regex::new(r"outdated.*checked").unwrap();
        assert!(!naked_response.text().unwrap().lines().any(|line| re.is_match(line)));
        assert!(param_response.text().unwrap().lines().any(|line| re.is_match(line)));

        let re = regex::Regex::new(r"problematic.*checked").unwrap();
        assert!(!naked_response.text().unwrap().lines().any(|line| re.is_match(line)));
        assert!(param_response.text().unwrap().lines().any(|line| re.is_match(line)));

        let re = regex::Regex::new(r"vulnerable.*checked").unwrap();
        assert!(!naked_response.text().unwrap().lines().any(|line| re.is_match(line)));
        assert!(param_response.text().unwrap().lines().any(|line| re.is_match(line)));

        let re = regex::Regex::new(r"has_related.*checked").unwrap();
        assert!(!naked_response.text().unwrap().lines().any(|line| re.is_match(line)));
        assert!(param_response.text().unwrap().lines().any(|line| re.is_match(line)));
    }
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_pagination_base(pool: PgPool) {
    let response = Request::new(pool.clone(), "/projects/").perform().await;
    assert!(response.text().unwrap().contains("pkg_barbar_"));
    assert!(response.text().unwrap().contains("pkg_foofoo_"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_pagination_from(pool: PgPool) {
    let response = Request::new(pool.clone(), "/projects/pkg_foo/").perform().await;
    assert!(!response.text().unwrap().contains("pkg_barbar_"));
    assert!(response.text().unwrap().contains("pkg_foofoo_"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_pagination_to(pool: PgPool) {
    let response = Request::new(pool.clone(), "/projects/..pkg_foo/").perform().await;
    assert!(response.text().unwrap().contains("pkg_barbar_"));
    assert!(!response.text().unwrap().contains("pkg_foofoo_"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search_base(pool: PgPool) {
    let response = Request::new(pool.clone(), "/projects/").perform().await;
    assert!(response.text().unwrap().contains("pkg_barbar_"));
    assert!(response.text().unwrap().contains("pkg_foofoo_"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search_a(pool: PgPool) {
    let response = Request::new(pool.clone(), "/projects/?search=bar").perform().await;
    assert!(response.text().unwrap().contains("pkg_barbar_"));
    assert!(!response.text().unwrap().contains("pkg_foofoo_"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_search_b(pool: PgPool) {
    let response = Request::new(pool.clone(), "/projects/?search=foo").perform().await;
    assert!(!response.text().unwrap().contains("pkg_barbar_"));
    assert!(response.text().unwrap().contains("pkg_foofoo_"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_repo_base(pool: PgPool) {
    let response = Request::new(pool, "/projects/").perform().await;
    assert!(response.text().unwrap().contains("pkg_12e"));
    assert!(response.text().unwrap().contains("pkg_24e"));
    assert!(response.text().unwrap().contains("pkg_1224e"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_inrepo(pool: PgPool) {
    let response = Request::new(pool, "/projects/?inrepo=ubuntu_12").perform().await;
    assert!(response.text().unwrap().contains("pkg_12e"));
    assert!(!response.text().unwrap().contains("pkg_24e"));
    assert!(response.text().unwrap().contains("pkg_1224e"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_notinrepo(pool: PgPool) {
    let response = Request::new(pool, "/projects/?notinrepo=ubuntu_12").perform().await;
    assert!(!response.text().unwrap().contains("pkg_12e"));
    assert!(response.text().unwrap().contains("pkg_24e"));
    assert!(!response.text().unwrap().contains("pkg_1224e"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_inrepo_newest(pool: PgPool) {
    let response = Request::new(pool, "/projects/?inrepo=ubuntu_12&newest=1").perform().await;
    assert!(response.text().unwrap().contains("pkg_1224_newest_12"));
    assert!(!response.text().unwrap().contains("pkg_1224_newest_24"));
}

#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("common_repositories", "projects_data"))]
async fn test_inrepo_outdated(pool: PgPool) {
    let response = Request::new(pool, "/projects/?inrepo=ubuntu_12&outdated=1").perform().await;
    assert!(!response.text().unwrap().contains("pkg_1224_newest_12"));
    assert!(response.text().unwrap().contains("pkg_1224_newest_24"));
}
