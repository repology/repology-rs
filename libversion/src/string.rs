//! # Low level ASCII char and string functions

/// Check if a byte is ASCII alphabetic character
pub fn is_alpha(c: u8) -> bool {
    c >= b'a' && c <= b'z' || c >= b'A' && c <= b'Z'
}

/// Check if a byte is ASCII digit
pub fn is_number(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Check if a byte is version compoment separator
pub fn is_separator(c: u8) -> bool {
    !is_alpha(c) && !is_number(c)
}

pub fn to_lower(c: u8) -> u8 {
    if c >= b'A' && c <= b'Z' {
        c - b'A' + b'a'
    } else {
        c
    }
}

pub fn string_is_equal_to_lowercase(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().map(|c| to_lower(c)).eq(b.bytes())
}

pub fn string_has_prefix_lowercase(s: &str, prefix: &str) -> bool {
    s.len() >= prefix.len() && string_is_equal_to_lowercase(&s[0..prefix.len()], prefix)
}

pub fn split_alpha(s: &str) -> (&str, &str) {
    let pos = s.bytes().position(|c| !is_alpha(c)).unwrap_or(s.len());
    (&s[0..pos], &s[pos..])
}

pub fn split_number(s: &str) -> (&str, &str) {
    let pos = s.bytes().position(|c| !is_number(c)).unwrap_or(s.len());
    (&s[0..pos], &s[pos..])
}

pub fn skip_zeroes(s: &str) -> &str {
    let pos = s.bytes().position(|c| c != b'0').unwrap_or(s.len());
    &s[pos..]
}

pub fn skip_separator(s: &str) -> &str {
    let pos = s.bytes().position(|c| !is_separator(c)).unwrap_or(s.len());
    &s[pos..]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_alpha() {
        assert!(is_alpha(b'a'));
        assert!(is_alpha(b'z'));
        assert!(is_alpha(b'A'));
        assert!(is_alpha(b'Z'));
        assert!(!is_alpha(b'0'));
        assert!(!is_alpha(b'.'));
        assert!(!is_alpha(b'-'));
        assert!(!is_alpha(b' '));
    }

    #[test]
    fn test_is_number() {
        assert!(is_number(b'0'));
        assert!(is_number(b'9'));
        assert!(!is_number(b'a'));
        assert!(!is_number(b'A'));
        assert!(!is_number(b'.'));
        assert!(!is_number(b'-'));
        assert!(!is_number(b' '));
    }

    #[test]
    fn test_is_separator() {
        assert!(is_separator(b'.'));
        assert!(is_separator(b'-'));
        assert!(is_separator(b' '));
        assert!(!is_separator(b'0'));
        assert!(!is_separator(b'9'));
        assert!(!is_separator(b'a'));
        assert!(!is_separator(b'z'));
        assert!(!is_separator(b'A'));
        assert!(!is_separator(b'Z'));
    }

    #[test]
    fn test_to_lower() {
        assert_eq!(to_lower(b'A'), b'a');
        assert_eq!(to_lower(b'Z'), b'z');
        assert_eq!(to_lower(b'a'), b'a');
        assert_eq!(to_lower(b'z'), b'z');
        assert_eq!(to_lower(b'0'), b'0');
        assert_eq!(to_lower(b'-'), b'-');
    }

    #[test]
    fn test_string_is_equal_to_lowercase() {
        assert!(string_is_equal_to_lowercase("foo", "foo"));
        assert!(string_is_equal_to_lowercase("FOO", "foo"));
        assert!(!string_is_equal_to_lowercase("foo", "bar"));
    }

    #[test]
    fn test_string_has_prefix_ci() {
        assert!(string_has_prefix_lowercase("foo", "foo"));
        assert!(string_has_prefix_lowercase("foobar", "foo"));
        assert!(string_has_prefix_lowercase("FOOBAR", "foo"));
        assert!(!string_has_prefix_lowercase("foo", "bar"));
        assert!(!string_has_prefix_lowercase("foobar", "bar"));
    }

    #[test]
    fn test_skip_zeroes() {
        assert_eq!(skip_zeroes("0001"), "1");
        assert_eq!(skip_zeroes("1000"), "1000");
        assert_eq!(skip_zeroes("123"), "123");
        assert_eq!(skip_zeroes("000"), "");
    }

    #[test]
    fn test_skip_separator() {
        assert_eq!(skip_separator("-1-"), "1-");
        assert_eq!(skip_separator("1-1"), "1-1");
        assert_eq!(skip_separator("---"), "");
        assert_eq!(skip_separator("abc"), "abc");
    }
}
