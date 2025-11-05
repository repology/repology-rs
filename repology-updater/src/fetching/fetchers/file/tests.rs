// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tokio::test]
async fn test_fetch() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/data.txt")
        .with_status(200)
        .with_body("Success")
        .create();

    let tmpdir = tempfile::tempdir().unwrap();
    let state_path = tmpdir.path().join("state");

    let fetcher = FileFetcher::new(FileFetcherOptions {
        url: server.url() + "/data.txt",
        ..Default::default()
    });
    let fetch_result = fetcher
        .fetch(&state_path, FetchPoliteness::default())
        .await
        .unwrap();

    mock.assert();
    assert!(fetch_result.was_modified);
    assert_eq!(
        std::fs::read_to_string(&fetch_result.state_path).unwrap(),
        "Success"
    );
    assert!(fetch_result.accept().await.is_ok());
}

#[tokio::test]
async fn test_fetch_fail() {
    let mut server = mockito::Server::new_async().await;
    let mock = server.mock("GET", "/data.txt").with_status(404).create();

    let tmpdir = tempfile::tempdir().unwrap();
    let state_path = tmpdir.path().join("state");

    let fetcher = FileFetcher::new(FileFetcherOptions {
        url: server.url() + "/data.txt",
        ..Default::default()
    });
    let fetch_result = fetcher.fetch(&state_path, FetchPoliteness::default()).await;

    mock.assert();
    assert!(fetch_result.is_err());
}

#[tokio::test]
async fn test_user_agent() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/data.txt")
        .with_status(200)
        .match_header(
            "user-agent",
            mockito::Matcher::Regex(r"repology-fetcher.*".to_string()),
        )
        .create();

    let tmpdir = tempfile::tempdir().unwrap();
    let state_path = tmpdir.path().join("state");

    FileFetcher::new(FileFetcherOptions {
        url: server.url() + "/data.txt",
        ..Default::default()
    })
    .fetch(&state_path, FetchPoliteness::default())
    .await
    .unwrap();

    mock.assert();
}

async fn fetch_with_compression(data: &[u8], compression: Compression) -> String {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/data")
        .with_status(200)
        .with_body(data)
        .create();

    let tmpdir = tempfile::tempdir().unwrap();
    let state_path = tmpdir.path().join("state");

    let fetcher = FileFetcher::new(FileFetcherOptions {
        url: server.url() + "/data",
        compression: Some(compression),
        ..Default::default()
    });
    let fetch_result = fetcher
        .fetch(&state_path, FetchPoliteness::default())
        .await
        .unwrap();

    std::fs::read_to_string(&fetch_result.state_path).unwrap()
}

#[tokio::test]
#[ignore]
async fn test_fetch_bz2() {
    let data = include_bytes!("fixtures/data.bz2");
    let result = fetch_with_compression(data, Compression::Bz2);
    assert_eq!(result.await.as_str(), "Success");
}

#[tokio::test]
#[ignore]
async fn test_fetch_gz() {
    let data = include_bytes!("fixtures/data.gz");
    let result = fetch_with_compression(data, Compression::Gz);
    assert_eq!(result.await.as_str(), "Success");
}

#[tokio::test]
#[ignore]
async fn test_fetch_xz() {
    let data = include_bytes!("fixtures/data.xz");
    let result = fetch_with_compression(data, Compression::Xz);
    assert_eq!(result.await.as_str(), "Success");
}

#[tokio::test]
#[ignore]
async fn test_fetch_zstd() {
    let data = include_bytes!("fixtures/data.zst");
    let result = fetch_with_compression(data, Compression::Zstd);
    assert_eq!(result.await.as_str(), "Success");
}

#[tokio::test]
async fn test_allow_zero_size() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/data")
        .with_status(200)
        .with_body("")
        .create();

    let tmpdir = tempfile::tempdir().unwrap();
    let state_path = tmpdir.path().join("state");

    let fetcher = FileFetcher::new(FileFetcherOptions {
        url: server.url() + "/data",
        allow_zero_size: false,
        ..Default::default()
    });
    let result = fetcher.fetch(&state_path, FetchPoliteness::default()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_timeout() {
    let mut server = mockito::Server::new_async().await;
    // no future or timeout support in mockito (https://github.com/lipanski/mockito/pull/163),
    // so we have to blocking sleep; hopefully it does not affect other tests much
    server
        .mock("GET", "/data")
        .with_status(200)
        .with_body_from_request(|_| {
            std::thread::sleep(Duration::from_millis(20));
            "success".into()
        })
        .create();

    let tmpdir = tempfile::tempdir().unwrap();
    let state_path = tmpdir.path().join("state");

    let fetcher = FileFetcher::new(FileFetcherOptions {
        url: server.url() + "/data",
        timeout: Some(Duration::from_millis(10)),
        ..Default::default()
    });
    let result = fetcher.fetch(&state_path, FetchPoliteness::default()).await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore]
async fn test_cache_buster() {
    let mut server = mockito::Server::new_async().await;
    // no future or timeout support in mockito (https://github.com/lipanski/mockito/pull/163),
    // so we have to blocking sleep; hopefully it does not affect other tests much
    server
        .mock("GET", "/data")
        .with_status(200)
        .with_body_from_request(|request| request.path_and_query().into())
        .create();

    let tmpdir = tempfile::tempdir().unwrap();
    let state_path = tmpdir.path().join("state");

    let fetcher = FileFetcher::new(FileFetcherOptions {
        url: server.url() + "/data?CACHE_BUSTER",
        cache_buster: Some("CACHE_BUSTER".into()),
        ..Default::default()
    });
    let fetch_result = fetcher
        .fetch(&state_path, FetchPoliteness::default())
        .await
        .unwrap();
    let data1 = std::fs::read_to_string(&fetch_result.state_path).unwrap();
    fetch_result.accept().await.unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    let fetch_result = fetcher
        .fetch(&state_path, FetchPoliteness::default())
        .await
        .unwrap();
    let data2 = std::fs::read_to_string(&fetch_result.state_path).unwrap();
    fetch_result.accept().await.unwrap();

    assert_ne!(data1, data2);
}

#[tokio::test]
async fn test_not_modified() {
    let mut server = mockito::Server::new_async().await;
    let mock1 = server
        .mock("GET", "/data")
        .with_status(200)
        .with_body("abc")
        .with_header("etag", "abcdef")
        .match_header("if-none-match", mockito::Matcher::Missing)
        .expect(2)
        .create();
    let mock2 = server
        .mock("GET", "/data")
        .with_status(304)
        .match_header("if-none-match", "abcdef")
        .create();

    let tmpdir = tempfile::tempdir().unwrap();
    let state_path = tmpdir.path().join("state");

    let fetcher = FileFetcher::new(FileFetcherOptions {
        url: server.url() + "/data",
        ..Default::default()
    });

    {
        let res = fetcher
            .fetch(&state_path, FetchPoliteness::default())
            .await
            .unwrap();
        assert!(res.was_modified);
        assert_eq!(
            std::fs::read_to_string(&res.state_path).unwrap(),
            "abc".to_string()
        );
        // res is not accepted!
    }

    {
        let res = fetcher
            .fetch(&state_path, FetchPoliteness::default())
            .await
            .unwrap();
        assert!(res.was_modified);
        assert_eq!(
            std::fs::read_to_string(&res.state_path).unwrap(),
            "abc".to_string()
        );
        res.accept().await.unwrap();
    }

    {
        let res = fetcher
            .fetch(&state_path, FetchPoliteness::default())
            .await
            .unwrap();
        assert!(!res.was_modified);
        assert_eq!(
            std::fs::read_to_string(&res.state_path).unwrap(),
            "abc".to_string()
        );
        res.accept().await.unwrap();
    }

    mock1.assert();
    mock2.assert();
}
