// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tokio::test]
async fn test_body_cityhash64() {
    let resp = perform_mock_request("aaa").await;
    assert_eq!(resp.body_cityhash64(), 0xeea159c5c8517ae9);
}

#[tokio::test]
async fn test_body_length() {
    let resp = perform_mock_request("aaa").await;
    assert_eq!(resp.body_length(), 3);
}
