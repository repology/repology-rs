// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;

use crate::result::EndpointResult;
use crate::state::AppState;

pub async fn static_file(
    Path(file_name): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> EndpointResult {
    let file = if let Some(file) = state.static_files.by_hashed_name(&file_name) {
        file
    } else {
        return Ok((StatusCode::NOT_FOUND, "not found".to_owned()).into_response());
    };

    let content_type = match file_name.rsplit_once(".").map(|(_, ext)| ext).unwrap_or("") {
        "css" => mime::TEXT_CSS.as_ref(),
        "ico" => "application/x-icon",
        "js" => mime::APPLICATION_JAVASCRIPT.as_ref(),
        "png" => mime::IMAGE_PNG.as_ref(),
        "svg" => mime::IMAGE_SVG.as_ref(),
        "txt" => mime::TEXT_PLAIN.as_ref(),
        "xml" => mime::TEXT_XML.as_ref(),
        _ => mime::APPLICATION_OCTET_STREAM.as_ref(),
    };

    let accepts_gzip = headers
        .get(header::ACCEPT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim())
                .any(|item| item == "gzip" || item.starts_with("gzip;"))
        })
        .unwrap_or(false);

    Ok(if accepts_gzip {
        (
            [
                (header::CONTENT_TYPE, HeaderValue::from_static(content_type)),
                (header::CONTENT_ENCODING, HeaderValue::from_static("gzip")),
                (
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=31536000, immutable"),
                ),
            ],
            file.compressed_content.clone(),
        )
            .into_response()
    } else {
        (
            [
                (header::CONTENT_TYPE, HeaderValue::from_static(content_type)),
                (
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=31536000, immutable"),
                ),
            ],
            file.original_content,
        )
            .into_response()
    })
}