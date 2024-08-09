// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub struct EndpointError(anyhow::Error);

impl IntoResponse for EndpointError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal server error: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for EndpointError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub type EndpointResult = Result<Response, EndpointError>;
