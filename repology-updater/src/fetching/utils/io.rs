// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io;
use std::path::Path;

use async_compression::tokio::bufread::{BzDecoder, GzipDecoder, XzDecoder, ZstdDecoder};
use futures_util::StreamExt;
use tokio::fs::File;
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;

use crate::fetching::utils::compression::Compression;

pub async fn save_http_stream_to_file(
    response: reqwest::Response,
    path: &Path,
    compression: Option<Compression>,
) -> anyhow::Result<u64> {
    let mut file = File::create(&path).await?;

    let stream = response.bytes_stream();
    let reader = StreamReader::new(stream.map(|r| r.map_err(io::Error::other)));

    let mut decoder: Box<dyn AsyncRead + Unpin + Send> = match compression {
        None => Box::new(reader),
        Some(Compression::Gz) => Box::new(GzipDecoder::new(reader)),
        Some(Compression::Xz) => Box::new(XzDecoder::new(reader)),
        Some(Compression::Bz2) => Box::new(BzDecoder::new(reader)),
        Some(Compression::Zstd) => Box::new(ZstdDecoder::new(reader)),
    };

    let num_bytes = tokio::io::copy(&mut decoder, &mut file).await?;

    file.sync_all().await?;

    Ok(num_bytes)
}
