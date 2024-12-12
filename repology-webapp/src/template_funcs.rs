// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use url::Url;

/// Given an url, try to extract domain for Qualys SSL Server Test
///
/// Qualys requires domain (not an IP address) and does not allow
/// custom ports, so only return host part if
pub fn extract_domain_for_ssltest(url: &str) -> Option<String> {
    let url = Url::parse(url).ok()?;
    if url.port_or_known_default().is_none_or(|port| port == 443) {
        url.domain().map(|domain| domain.into())
    } else {
        None
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    mod test_extract_domain_for_ssltest {
        use super::super::*;

        #[test]
        fn test_basic() {
            assert_eq!(
                extract_domain_for_ssltest("https://example.com/").as_deref(),
                Some("example.com")
            );
        }

        #[test]
        fn test_default_port() {
            assert_eq!(
                extract_domain_for_ssltest("https://example.com:443/").as_deref(),
                Some("example.com")
            );
        }

        #[test]
        fn test_custom_port() {
            assert_eq!(extract_domain_for_ssltest("https://example.com:444/"), None);
        }

        #[test]
        fn test_custom_schema() {
            assert_eq!(
                extract_domain_for_ssltest("git+https://example.com/").as_deref(),
                Some("example.com")
            );
        }

        #[test]
        fn test_not_https() {
            assert_eq!(extract_domain_for_ssltest("http://example.com/"), None);
        }

        #[test]
        fn test_invalid_urls() {
            assert_eq!(extract_domain_for_ssltest(""), None);
            assert_eq!(extract_domain_for_ssltest("!&?"), None);
            assert_eq!(extract_domain_for_ssltest(" "), None);
            assert_eq!(extract_domain_for_ssltest("..."), None);
            assert_eq!(extract_domain_for_ssltest("\n"), None);
            assert_eq!(extract_domain_for_ssltest("example.com"), None);
            assert_eq!(extract_domain_for_ssltest("file:///example.com"), None);
        }
    }
}
