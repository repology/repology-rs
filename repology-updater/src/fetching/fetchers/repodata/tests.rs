// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tokio::test]
async fn test_fetch() {
    let mut server = mockito::Server::new_async().await;
    let repomd_mock = server
        .mock("GET", "/foo/repodata/repomd.xml")
        .with_status(200)
        .with_body(include_bytes!("fixtures/repomd.xml"))
        .create();
    let primary_mock = server
        .mock("GET", "/foo/repodata/b17f53f807329932851a689e7f8bbc065ab38f71dc213941daedfa4095bbc72a-primary.xml.xz")
        .with_status(200)
        .with_body(include_bytes!("fixtures/primary.xml.xz"))
        .create();

    let tmpdir = tempfile::tempdir().unwrap();
    let state_path = tmpdir.path().join("state");

    let fetcher = RepodataFetcher::new(RepodataFetcherOptions {
        url: server.url() + "/foo",
        ..Default::default()
    });
    let fetch_result = fetcher.fetch(&state_path, &Http::default()).await.unwrap();

    repomd_mock.assert();
    primary_mock.assert();
    assert_eq!(
        std::fs::read_to_string(&fetch_result.state_path).unwrap(),
        "...xml..."
    );
    assert!(fetch_result.accept().await.is_ok());
}

#[tokio::test]
async fn test_not_modified() {
    let mut server = mockito::Server::new_async().await;
    let repomd_mock = server
        .mock("GET", "/foo/repodata/repomd.xml")
        .with_status(200)
        .with_body(include_bytes!("fixtures/repomd.xml"))
        .expect(3)
        .create();
    let primary_mock = server
        .mock("GET", "/foo/repodata/b17f53f807329932851a689e7f8bbc065ab38f71dc213941daedfa4095bbc72a-primary.xml.xz")
        .with_status(200)
        .with_body(include_bytes!("fixtures/primary.xml.xz"))
        .expect(2)
        .create();

    let tmpdir = tempfile::tempdir().unwrap();
    let state_path = tmpdir.path().join("state");

    {
        let res = RepodataFetcher::new(RepodataFetcherOptions {
            url: server.url() + "/foo",
            ..Default::default()
        })
        .fetch(&state_path, &Http::default())
        .await
        .unwrap();

        assert!(res.was_modified);
        assert_eq!(
            std::fs::read_to_string(&res.state_path).unwrap(),
            "...xml..."
        );
        // not accepted
    }

    {
        let res = RepodataFetcher::new(RepodataFetcherOptions {
            url: server.url() + "/foo",
            ..Default::default()
        })
        .fetch(&state_path, &Http::default())
        .await
        .unwrap();

        assert!(res.was_modified);
        assert_eq!(
            std::fs::read_to_string(&res.state_path).unwrap(),
            "...xml..."
        );
        res.accept().await.unwrap();
    }

    {
        let res = RepodataFetcher::new(RepodataFetcherOptions {
            url: server.url() + "/foo",
            ..Default::default()
        })
        .fetch(&state_path, &Http::default())
        .await
        .unwrap();

        assert!(!res.was_modified);
        assert_eq!(
            std::fs::read_to_string(&res.state_path).unwrap(),
            "...xml..."
        );
        res.accept().await.unwrap();
    }

    repomd_mock.assert();
    primary_mock.assert();
}

#[tokio::test]
async fn test_mirror_list() {
    let mut server = mockito::Server::new_async().await;
    let mirrorlist_mock = server
        .mock("GET", "/foo/mirror.list")
        .with_status(200)
        .with_body(format!("{}/foo/\nhttps://example.com/foo", server.url()))
        .create();
    let repomd_mock = server
        .mock("GET", "/foo/repodata/repomd.xml")
        .with_status(200)
        .with_body(include_bytes!("fixtures/repomd.xml"))
        .create();
    let primary_mock = server
        .mock("GET", "/foo/repodata/b17f53f807329932851a689e7f8bbc065ab38f71dc213941daedfa4095bbc72a-primary.xml.xz")
        .with_status(200)
        .with_body(include_bytes!("fixtures/primary.xml.xz"))
        .create();

    let tmpdir = tempfile::tempdir().unwrap();
    let state_path = tmpdir.path().join("state");

    let fetcher = RepodataFetcher::new(RepodataFetcherOptions {
        url: server.url() + "/foo/mirror.list",
        ..Default::default()
    });
    let fetch_result = fetcher.fetch(&state_path, &Http::default()).await.unwrap();

    mirrorlist_mock.assert();
    repomd_mock.assert();
    primary_mock.assert();
    assert_eq!(
        std::fs::read_to_string(&fetch_result.state_path).unwrap(),
        "...xml..."
    );
    assert!(fetch_result.accept().await.is_ok());
}
