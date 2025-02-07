// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

use serde_json::json;

#[tokio::test]
async fn test_json() {
    let resp = perform_mock_request(r#"{"foo":"bar"}"#).await;
    assert_eq!(resp.json().unwrap(), json!({"foo":"bar"}));
}
