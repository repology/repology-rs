// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tokio::test]
async fn test_text() {
    let resp = perform_mock_request("abc").await;
    assert_eq!(resp.text().unwrap(), "abc");
    assert_eq!(resp.text(), Ok("abc"));
}
