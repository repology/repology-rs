// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use askama::filters::Safe;
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

/// Format a number into a short string representation
///
/// If the number is greater than or equal to 1,000, it will be
/// represented in thousands with a "k" suffix (e.g. 1,200 → "1k").
/// Otherwise, it will be returned as a plain string.
pub fn format_number_short(number: &i32) -> Safe<String> {
    if *number >= 1_000 {
        let short_value = *number as f32 / 1_000.0;
        Safe(format!(
            r#"<span title="{}">{:.2}k</span>"#,
            *number, short_value
        ))
    } else {
        Safe((*number).to_string())
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

    #[test]
    fn test_format_number_short() {
        assert_eq!(
            super::format_number_short(&12345).0,
            "<span title=\"12345\">12.35k</span>".to_owned()
        );
        assert_eq!(
            super::format_number_short(&2345).0,
            "<span title=\"2345\">2.35k</span>".to_owned()
        );
        assert_eq!(
            super::format_number_short(&1500).0,
            "<span title=\"1500\">1.50k</span>".to_owned()
        );
        assert_eq!(
            super::format_number_short(&1001).0,
            "<span title=\"1001\">1.00k</span>".to_owned()
        );
        assert_eq!(super::format_number_short(&999).0, "999".to_owned());
        assert_eq!(
            super::format_number_short(&1000).0,
            "<span title=\"1000\">1.00k</span>".to_owned()
        );
        assert_eq!(super::format_number_short(&171).0, "171".to_owned());
        assert_eq!(super::format_number_short(&0).0, "0".to_owned());
    }
}
