// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tokio::test]
async fn test_as_snapshot() {
    let resp = perform_mock_request("abc").await;
    insta::assert_snapshot!(resp.as_snapshot().unwrap());
}
