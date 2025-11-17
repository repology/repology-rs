// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;
use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};

use repology_common::PackageFlags;

// allows alpha1, alpha20210101, alpha.1, but not alpha.20210101 (which is parsed as alpha instead)
static PRERELEASE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"^(.*?)((?:alpha|beta|rc)(?:[0-9]+|\.[0-9]{1,2})?)((?:[^0-9].*)?)$")
        .case_insensitive(true)
        .build()
        .unwrap()
});
static PRERELEASE_FALLBACK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"^(.*?)((?:dev|pre)(?:[0-9]+|\.[0-9]{1,2})?)((?:[^0-9].*)?)$")
        .case_insensitive(true)
        .build()
        .unwrap()
});
static POSTRELEASE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"^(.*?)((?:post)[0-9]+)(.*)$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

static SNAPSHOT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // XXX: try changing 20[0-9]{6} → [0-9]{7} for catching all-numeric commit hashes too
    RegexBuilder::new(r"[a-z]|20[0-9]{6}")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub fn normalize_rpm_version<'a>(
    version: &'a str,
    release: &str,
    disttags: &[impl AsRef<str>],
) -> (Cow<'a, str>, PackageFlags) {
    let mut current_version = Cow::from(version);
    let mut flags = PackageFlags::empty();
    let mut cleaned_up_release = String::with_capacity(release.len());

    // Remove tags (e.g. `el`, `fc`, `mga`) from the release field as we don't want
    // these to interfere with normalization, splitting the release into parts
    // XXX: Will just replacing these with a separator be less obscure and more effective?
    let mut parts = vec![release];
    for tag in disttags {
        let tag: &str = tag.as_ref();
        parts = parts.into_iter().flat_map(|part| part.split(tag)).collect()
    }

    // Try to extract known-valid pre-release (such as `alpha2`, `beta.3`, `pre11`)
    // and post-release (such as `post3`) version suffixes from RPM __release__ field
    // and append these to output version; for pre-release suffixes, set Devel flag
    // as well
    for part in parts {
        if let Some(caps) = PRERELEASE_REGEX
            .captures(part)
            .or_else(|| PRERELEASE_FALLBACK_REGEX.captures(part))
            .inspect(|_| {
                // adds Devel flag if any of PRERELEASE_*_REGEX has matched
                flags = PackageFlags::Devel;
            })
            .or_else(|| POSTRELEASE_REGEX.captures(part))
        {
            let (left, version_suffix, right) = (&caps[1], &caps[2], &caps[3]);

            // legal prerelease or postrelease match
            current_version.to_mut().push('-');
            current_version.to_mut().push_str(version_suffix);

            cleaned_up_release.push_str(left);
            if !left.is_empty() && !right.is_empty() {
                cleaned_up_release.push('.');
            }
            cleaned_up_release.push_str(right);
        } else {
            cleaned_up_release.push_str(part);
        }
    }

    // RPM __release__ starting with zero is supposed to mean pre-release.
    // If so, but we've failed to extract known pre-release suffix, mark
    // version as ignored, as it's likely some kind of a snapshot.
    if (cleaned_up_release == "0" || cleaned_up_release.starts_with("0."))
        && !flags.contains(PackageFlags::Devel)
    {
        flags |= PackageFlags::Ignore;
    }

    // If the release contains alphabetic characters (may be commit hashes,
    // references to VCS or other obscure elements) or dates, assume it's a
    // snapshot regardless
    if SNAPSHOT_REGEX.is_match(&cleaned_up_release) {
        flags |= PackageFlags::Ignore;
    }

    (current_version, flags)
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use repology_common::PackageFlags as Pf;

    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            normalize_rpm_version("1.2.3", "1", &["el"]),
            ("1.2.3".into(), PackageFlags::empty())
        );
    }

    #[test]
    fn test_release_starts_with_zero() {
        // Release starts with zero - potentially prerelease or a snapshot
        assert_eq!(
            normalize_rpm_version("1.2.3", "0", &["el"]),
            ("1.2.3".into(), Pf::Ignore)
        );
    }

    #[test]
    fn test_release_suggests_snapshot() {
        // Release suggests snapshot, even if it doesn't start with zero
        assert_eq!(
            normalize_rpm_version("1.2.3", "1.20200101", &["el"]),
            ("1.2.3".into(), Pf::Ignore)
        );
        assert_eq!(
            normalize_rpm_version("1.2.3", "1.git1234567", &["el"]),
            ("1.2.3".into(), Pf::Ignore)
        );
    }

    #[test]
    fn test_release_contains_good_prerelease() {
        for keyword in &["alpha", "beta", "rc", "dev", "pre"] {
            for prefix_sep in &["", "."] {
                for infix_sep in &["", "."] {
                    let suffix = format!("{prefix_sep}{keyword}{infix_sep}1");
                    let expected_suffix = format!("-{keyword}{infix_sep}1");
                    assert_eq!(
                        normalize_rpm_version("1.2.3", &format!("0{suffix}"), &["el"]),
                        (format!("1.2.3{expected_suffix}").into(), Pf::Devel)
                    );
                    assert_eq!(
                        normalize_rpm_version("1.2.3", &format!("1{suffix}"), &["el"]),
                        (format!("1.2.3{expected_suffix}").into(), Pf::Devel)
                    );
                }
            }
        }
    }

    #[test]
    fn test_release_contains_good_postrelease() {
        assert_eq!(
            normalize_rpm_version("1.2.3", "0post1", &["el"]),
            ("1.2.3-post1".into(), Pf::Ignore)
        );
        assert_eq!(
            normalize_rpm_version("1.2.3", "0.post1", &["el"]),
            ("1.2.3-post1".into(), Pf::Ignore)
        );
        assert_eq!(
            normalize_rpm_version("1.2.3", "1post1", &["el"]),
            ("1.2.3-post1".into(), Pf::empty())
        );
        assert_eq!(
            normalize_rpm_version("1.2.3", "1.post1", &["el"]),
            ("1.2.3-post1".into(), Pf::empty())
        );
    }

    #[test]
    fn test_release_prerelease_without_number() {
        assert_eq!(
            normalize_rpm_version("1.2.3", "0alpha", &["el"]),
            ("1.2.3-alpha".into(), Pf::Devel)
        );
        assert_eq!(
            normalize_rpm_version("1.2.3", "0.alpha", &["el"]),
            ("1.2.3-alpha".into(), Pf::Devel)
        );
        assert_eq!(
            normalize_rpm_version("1.2.3", "1alpha", &["el"]),
            ("1.2.3-alpha".into(), Pf::Devel)
        );
        assert_eq!(
            normalize_rpm_version("1.2.3", "1.alpha", &["el"]),
            ("1.2.3-alpha".into(), Pf::Devel)
        );
    }

    #[test]
    fn test_release_prerelease_dot_longnumber() {
        assert_eq!(
            normalize_rpm_version("1.2.3", "0.alpha20210101", &["el"]),
            ("1.2.3-alpha20210101".into(), Pf::Devel)
        );
        assert_eq!(
            normalize_rpm_version("1.2.3", "0.alpha.20210101", &["el"]),
            ("1.2.3-alpha".into(), Pf::Devel | Pf::Ignore)
        );
        assert_eq!(
            normalize_rpm_version("1.2.3", "1.alpha20210101", &["el"]),
            ("1.2.3-alpha20210101".into(), Pf::Devel)
        );
        assert_eq!(
            normalize_rpm_version("1.2.3", "1.alpha.20210101", &["el"]),
            ("1.2.3-alpha".into(), Pf::Devel | Pf::Ignore)
        );
    }

    #[test]
    fn test_release_tag() {
        // Release tags, if specified for the repo, are not condidered as
        // a sign of a snapshot and do not produce Ignore flag
        assert_eq!(
            normalize_rpm_version("1.2.3", "1.el6", &["el"]),
            ("1.2.3".into(), Pf::empty())
        );
        assert_eq!(
            normalize_rpm_version("1.2.3", "1.6el", &["el"]),
            ("1.2.3".into(), Pf::empty())
        );
    }

    #[test]
    fn test_release_tag_multi_match() {
        assert_eq!(
            normalize_rpm_version("1.2.3", "1.mga1.mga2", &["mga"]),
            ("1.2.3".into(), Pf::empty())
        );
    }

    #[test]
    fn test_release_tag_multi_tag() {
        assert_eq!(
            normalize_rpm_version("1.2.3", "1.el1", &["mga", "el", "fc"]),
            ("1.2.3".into(), Pf::empty())
        );
    }

    #[test]
    fn test_release_tag_glued() {
        // Removed tags should not corrupt prerelease versions
        assert_eq!(
            normalize_rpm_version("1.2.3", "1beta3el6", &["el"]),
            ("1.2.3-beta3".into(), Pf::Devel)
        );
    }

    #[test]
    fn test_real_world_asciidoctor() {
        assert_eq!(
            normalize_rpm_version("1.5.0", "0.2.alpha.13.fc26", &["fc"]),
            ("1.5.0-alpha.13".into(), Pf::Devel)
        );
    }

    #[test]
    fn test_real_world_roundcube() {
        assert_eq!(
            normalize_rpm_version("1.5", "0.beta.2.mga8", &["mga"]),
            ("1.5-beta.2".into(), Pf::Devel)
        );
    }

    #[test]
    fn test_real_world_aeskulap() {
        assert_eq!(
            normalize_rpm_version("0.2.2", "0.36.beta2.fc29", &["fc"]),
            ("0.2.2-beta2".into(), Pf::Devel)
        );
    }

    #[test]
    fn test_real_world_kumir() {
        // arguable: we parse out devel suffix from Release, but set Ignore flag
        // because Release also contains signs of snapshot; instead, we could ignore
        // the latter and consider that the version with prerelease suffix is precise
        // enough to not be a pre-snapshot
        assert_eq!(
            normalize_rpm_version("2.1.0", "0.rc9.20190320.7.mga7", &["mga"]),
            ("2.1.0-rc9".into(), Pf::Ignore | Pf::Devel)
        );
    }

    #[test]
    fn test_real_world_novacom_client() {
        assert_eq!(
            normalize_rpm_version("1.1.0", "0.4.rc1.git.ff7641193a.el6", &["el"]),
            ("1.1.0-rc1".into(), Pf::Ignore | Pf::Devel)
        );
    }

    #[test]
    fn test_real_world_airstrike() {
        // arguable: we parse out pre6, but we don't expect it to be pre6a so we also ignore it
        assert_eq!(
            normalize_rpm_version("1.0", "1.0-0.pre6a.8.mga8", &["mga"]),
            ("1.0-pre6".into(), Pf::Ignore | Pf::Devel)
        );
    }

    #[test]
    #[ignore = "not handled - invalid prerelease keyword; even if we allow it, but there will not be correct ordering between prealpha/alpha/beta/..."]
    fn test_real_world_pocketsphinx() {
        assert_eq!(
            normalize_rpm_version("0.9", "0.5prealpha", &["el"]),
            ("0.9-prealpha".into(), Pf::Devel)
        );
    }

    #[test]
    #[ignore = "not handled - there's no knowing that `.2` is in fact not related to `beta`, and `beta.2` is much more likely intention (XXX: recheck this claim)"]
    fn test_real_world_php_pear_console_progressbar() {
        assert_eq!(
            normalize_rpm_version("0.5.2", "0.beta.2", &["el"]),
            ("0.5.2-beta".into(), Pf::Devel)
        );
    }

    #[test]
    fn test_real_world_xz() {
        assert_eq!(
            normalize_rpm_version("4.999.9", "0.5.beta.20091007git.el6", &["el"]),
            ("4.999.9-beta".into(), Pf::Devel | Pf::Ignore)
        );
    }

    #[test]
    #[ignore = "not handled, invalid prerelease keyword"]
    fn test_real_world_remmina() {
        assert_eq!(
            normalize_rpm_version("1.2.0", "5.rcgit.15.2", &["el"]),
            ("1.2.0-rcgit.15".into(), Pf::Devel | Pf::Ignore)
        );
    }

    #[test]
    fn test_real_world_icedtea_web() {
        // prefer standard suffixes like alpha|beta|rc over pre|dev
        assert_eq!(
            normalize_rpm_version("2.0.0", "pre.0.3.alpha16.patched1.1.fc35", &["fc"]),
            ("2.0.0-alpha16".into(), Pf::Devel | Pf::Ignore)
        );
    }

    #[test]
    fn test_real_world_opencontainers_runc() {
        assert_eq!(
            normalize_rpm_version("1.0.0", "0.rc92.7.dev.gitff819c7.mga8", &["mga"]),
            ("1.0.0-rc92".into(), Pf::Devel | Pf::Ignore)
        );
    }

    #[test]
    fn test_cow() {
        assert!(Cow::is_borrowed(
            &normalize_rpm_version("1.0.0", "0.mga8", &["mga"]).0
        ));
    }

    extern crate test;
    use test::Bencher;
    use test::black_box;

    #[bench]
    fn bench_simple(b: &mut Bencher) {
        b.iter(|| normalize_rpm_version(black_box("1.2.3"), black_box("0"), &[] as &[&str]));
    }

    #[bench]
    fn bench_long(b: &mut Bencher) {
        b.iter(|| {
            normalize_rpm_version(
                black_box("1.2.3"),
                black_box("pre.0.3.alpha16.patched1.1.fc35"),
                &["fc"],
            )
        });
    }

    #[bench]
    fn bench_many_tags(b: &mut Bencher) {
        b.iter(|| {
            normalize_rpm_version(
                black_box("1.2.3"),
                black_box("1.0-0.pre6a.8.mga8"),
                &["fc", "mga", "foo", "bar"],
            )
        });
    }
}
