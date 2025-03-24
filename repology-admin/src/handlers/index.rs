// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::response::{IntoResponse, Redirect};

use crate::result::EndpointResult;

pub async fn handle_index() -> EndpointResult {
    Ok(Redirect::to("/reports/new").into_response())
}
