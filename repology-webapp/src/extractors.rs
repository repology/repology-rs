// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::convert::Infallible;
use std::net::IpAddr;

use anyhow::Result;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use tracing::{error, info};

/// axum extractor for all possible client IP addresses
///
/// Depending on how the [repology] application is deployed, client address
/// needs to be extracted from different places. If application is directly
/// accessible to clients, client IP comes from tokio connection. If an application
/// is behind a reverse proxy, client address is usually passed in some HTTP header
/// such as X-Real-IP. There may be multiple headers, and a header may contain
/// multiple addresses, and some headers may also be spoofed.
///
/// For the sake of client identification or load balancing, this may require
/// application to know how it is deployed, to extract the address specifically
/// from the proper place. However, for the sake of client blacklising, we can
/// just extract all addresses from all possible locations, and avoid configuration.
///
/// Therefore, we use custom extractor and not https://docs.rs/axum-client-ip.
///
/// The current implementation does not really support ALL possible locations,
/// good enough job for how repology is currently deployed. If someone deploys
/// repology another way, and suffers from spam problems, this implementation
/// may be extended.
#[derive(Debug)]
pub struct PossibleClientAddresses(pub Vec<IpAddr>);

impl<S> FromRequestParts<S> for PossibleClientAddresses
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        const SOURCE_HEADERS: &[&str] = &["x-real-ip", "x-forwarded-for"];

        fn parse_address(data: &[u8]) -> Result<IpAddr> {
            Ok(std::str::from_utf8(data)?.trim().parse()?)
        }

        let mut addresses: Vec<IpAddr> = vec![];

        for header in SOURCE_HEADERS {
            let Some(value) = parts.headers.get(*header) else {
                continue;
            };
            // we iterate byte slice to avoid the possibility of malicious user poisoning
            // the whole header by injecting invalid utf sequence in previous header content
            for part in value.as_bytes().split(|b| *b == b',') {
                match parse_address(part) {
                    Ok(address) => addresses.push(address),
                    Err(err) => {
                        let part = String::from_utf8_lossy(part);
                        error!(%part, %err, "unable to parse client address");
                    }
                }
            }
        }
        info!(?parts.headers, "headers dump in PossibleClientAddresses extractor");
        Ok(Self(addresses))
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use std::net::Ipv4Addr;

    use http::HeaderValue;

    use super::*;

    #[tokio::test]
    async fn test_simple() {
        let mut parts = http::Request::builder()
            .header(
                "X-Real-IP",
                HeaderValue::from_bytes(b" 10.0.0.1 , 10.0.0.2 , garbage, \xff").unwrap(),
            )
            .header(
                "X-Forwarded-For",
                HeaderValue::from_bytes(b" 10.1.0.1 , 10.1.0.2 , garbage, \xff").unwrap(),
            )
            .body(())
            .unwrap()
            .into_parts()
            .0;

        assert_eq!(
            PossibleClientAddresses::from_request_parts(&mut parts, &())
                .await
                .unwrap()
                .0,
            vec![
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
                IpAddr::V4(Ipv4Addr::new(10, 1, 0, 1)),
                IpAddr::V4(Ipv4Addr::new(10, 1, 0, 2)),
            ]
        );
    }
}
