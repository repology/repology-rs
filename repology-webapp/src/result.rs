// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::error;

pub struct EndpointError(anyhow::Error);

impl IntoResponse for EndpointError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            if cfg!(debug_assertions) {
                format!("Internal server error:\n{:#?}", self.0)
            } else {
                format!("Internal server error")
            },
        )
            .into_response()
    }
}

impl<E> From<E> for EndpointError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let err = err.into();
        error!("{:#?}", err);
        Self(err)
    }
}

pub type EndpointResult = Result<Response, EndpointError>;
