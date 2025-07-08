// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;

use regex::Regex;

pub fn extract_maintainers(maintainers: &str) -> Vec<String> {
    // Note: bench below shows that compiling a regexp on each call is
    // 3x faster than using LazyCell/LazyLock. Is this true?
    let looks_like_email = Regex::new(r"^[^<>\s]+@[^<>\s]+$").unwrap();

    let mut emails = HashSet::new();

    for part in maintainers.split(',') {
        let mut has_other_words = false;
        let mut candidates = HashSet::new();
        for word in part.split_whitespace() {
            if let Some(word) = word
                .strip_prefix('<')
                .and_then(|word| word.strip_suffix('>'))
            {
                // email-looking thing in angle brackets is most likely email,
                // use it right away
                if looks_like_email.is_match(word) {
                    emails.insert(word.to_ascii_lowercase());
                }
            } else if looks_like_email.is_match(word) {
                // email-looking thing without brackets may as well be a part
                // of an obfuscated string, so only use it when there no other
                // words in the same part with it
                candidates.insert(word.to_ascii_lowercase());
            } else {
                has_other_words = true;
            }
        }

        if !has_other_words {
            emails.extend(candidates);
        }
    }

    let mut emails: Vec<_> = emails.into_iter().collect();
    emails.sort();
    emails
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        assert_eq!(
            extract_maintainers("amdmi3@FreeBSD.org"),
            vec!["amdmi3@freebsd.org".to_string()]
        );
        assert_eq!(
            extract_maintainers("amdmi3@FAKE"),
            vec!["amdmi3@fake".to_string()]
        );
    }

    #[test]
    fn test_name() {
        assert_eq!(
            extract_maintainers("Dmitry Marakasov <amdmi3@FreeBSD.org>"),
            vec!["amdmi3@freebsd.org".to_string()]
        );
        assert_eq!(
            extract_maintainers(r#""Dmitry Marakasov" <amdmi3@FreeBSD.org>"#),
            vec!["amdmi3@freebsd.org".to_string()]
        );
    }

    #[test]
    fn test_name_comma() {
        assert_eq!(
            extract_maintainers("Marakasov, Dmitry <amdmi3@FreeBSD.org>"),
            vec!["amdmi3@freebsd.org".to_string()]
        );
        assert_eq!(
            extract_maintainers(r#""Marakasov, Dmitry" <amdmi3@FreeBSD.org>"#),
            vec!["amdmi3@freebsd.org".to_string()]
        );
    }

    #[test]
    fn test_lists() {
        assert_eq!(
            extract_maintainers("amdmi3@FreeBSD.org,gnome@FreeBSD.org"),
            vec![
                "amdmi3@freebsd.org".to_string(),
                "gnome@freebsd.org".to_string()
            ]
        );
        assert_eq!(
            extract_maintainers("amdmi3@FreeBSD.org, gnome@FreeBSD.org"),
            vec![
                "amdmi3@freebsd.org".to_string(),
                "gnome@freebsd.org".to_string()
            ]
        );
        assert_eq!(
            extract_maintainers("amdmi3@FreeBSD.org gnome@FreeBSD.org"),
            vec![
                "amdmi3@freebsd.org".to_string(),
                "gnome@freebsd.org".to_string()
            ]
        );
    }

    #[test]
    fn test_list_name() {
        assert_eq!(
            extract_maintainers(
                "Dmitry Marakasov <amdmi3@FreeBSD.org>, Gnome Guys <gnome@FreeBSD.org>"
            ),
            vec![
                "amdmi3@freebsd.org".to_string(),
                "gnome@freebsd.org".to_string()
            ]
        );
    }

    #[test]
    fn test_list_name_complex() {
        assert_eq!(
            extract_maintainers(
                "Marakasov, Dmitry <amdmi3@FreeBSD.org>, Guys, Gnome <gnome@FreeBSD.org>"
            ),
            vec![
                "amdmi3@freebsd.org".to_string(),
                "gnome@freebsd.org".to_string()
            ]
        );
    }

    #[test]
    fn test_list_name_ambigous() {
        // Unlike samples in test_lists(), this sample is ambiguous
        // as words may denote either name, or an obfuscated email
        // So we expect these emails to be ignored as there's no
        // guarantee these are parsed correctly
        assert_eq!(
            extract_maintainers("dmitry marakasov amdmi3@FreeBSD.org, foo dot bar@FreeBSD.org"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn test_garbage() {
        assert_eq!(
            extract_maintainers(",amdmi3@FreeBSD.org, ,,   "),
            vec!["amdmi3@freebsd.org".to_string()]
        );
    }

    #[test]
    fn test_immune_to_obfuscation() {
        assert!(extract_maintainers("amdmi3[at]FreeBSD[dot]org").is_empty(),);
        assert!(extract_maintainers("amdmi3 [ at ] FreeBSD [ dot ] org").is_empty(),);
        assert!(extract_maintainers("amdmi3 at FreeBSD dot org").is_empty(),);
        assert!(extract_maintainers("amdmi3_at_FreeBSD.org").is_empty(),);
        assert!(extract_maintainers("amdmi3{at}FreeBSD{dot}org").is_empty(),);
        assert!(extract_maintainers("amdmi3 <at> freebsd {dot} org").is_empty(),);
        assert!(extract_maintainers("amdmi3~at~freebsd~dot~org").is_empty(),);
        assert!(extract_maintainers("amdmi3 (at) freebsd (dot) org").is_empty(),);
        assert!(extract_maintainers("amdmi3 __at__ freebsd __dot__ org").is_empty(),);
        assert!(extract_maintainers("amdmi3-at-freebsd-dot-org").is_empty(),);
        assert!(extract_maintainers("amdmi3<at>freebsd.org").is_empty(),);
        assert!(extract_maintainers("amdmi3 <at> freebsd.org").is_empty(),);
        assert!(extract_maintainers("amdmi3 [underscore] ports [at] freebsd.org").is_empty(),);
        assert!(extract_maintainers("amdmi3 plus ports@freebsd.org").is_empty(),);
        assert!(extract_maintainers("agent smith (amdmi3@freebsd.org)").is_empty(),);

        assert!(extract_maintainers("amdNOmi3@freeSPAMbsd.org (remove NO and SPAM)").is_empty(),);
        assert!(extract_maintainers("amdmi3 @ google mail").is_empty(),);
    }

    #[test]
    fn test_empty() {
        assert!(extract_maintainers("somecrap").is_empty());
        assert!(extract_maintainers("").is_empty());
        assert!(extract_maintainers("http://repology.org").is_empty());
        assert!(extract_maintainers("Repology <http://repology.org>").is_empty());
        assert!(extract_maintainers("nobody <really>").is_empty());
    }

    #[test]
    fn test_nonascii() {
        // not sure how this should be handled, but let's at least fix the behavior
        assert_eq!(
            extract_maintainers("ПЕТЯ@ВАСЯ"),
            vec!["ПЕТЯ@ВАСЯ".to_string()]
        );
    }

    extern crate test;
    use test::Bencher;
    use test::black_box;

    #[bench]
    fn bench_extract_maintainers(b: &mut Bencher) {
        b.iter(|| extract_maintainers(black_box("Dmitry Marakasov <amdmi3@FreeBSD.org>")));
    }
}
