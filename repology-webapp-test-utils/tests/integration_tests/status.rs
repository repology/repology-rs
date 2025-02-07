// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tokio::test]
async fn test_status() {
    let resp = perform_mock_request("aaa").await;
    assert_eq!(resp.status(), http::StatusCode::OK);
}
