// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::anyhow;

#[derive(Debug, PartialEq, Eq)]
pub struct Nevra<'a> {
    pub name: &'a str,
    pub epoch: Option<&'a str>,
    pub version: &'a str,
    pub release: &'a str,
    pub architecture: &'a str,
}

impl<'a> Nevra<'a> {
    pub fn try_parse(input: &'a str) -> Option<Nevra<'a>> {
        let input = input.trim_suffix(".rpm");

        let (input, architecture) = input.rsplit_once('.')?;
        let (input, release) = input.rsplit_once('-')?;
        let (name, version) = input.rsplit_once('-')?;

        let (epoch, version) = version
            .rsplit_once(':')
            .map(|(epoch, version)| (Some(epoch), version))
            .unwrap_or((None, version));

        Some(Self {
            name,
            epoch,
            version,
            release,
            architecture,
        })
    }

    pub fn parse(input: &'a str) -> anyhow::Result<Nevra<'a>> {
        Self::try_parse(input).ok_or_else(|| anyhow!("cannot parse NEVRA value ({})", input))
    }
}

pub fn merge_evr(epoch: Option<&str>, version: &str, release: &str) -> String {
    let mut res = String::with_capacity(
        epoch.map(|epoch| epoch.len() + 1).unwrap_or_default() + version.len() + 1 + release.len(),
    );

    if let Some(epoch) = epoch
        && epoch != "0"
    {
        res.push_str(epoch);
        res.push(':');
    }

    res.push_str(version);
    res.push('-');
    res.push_str(release);

    res
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_nevra_basic() {
        assert_eq!(
            Nevra::parse("foo-1.2.3-1.i386").unwrap(),
            Nevra {
                name: "foo",
                epoch: None,
                version: "1.2.3",
                release: "1",
                architecture: "i386"
            }
        );
    }

    #[test]
    fn test_nevra_ext() {
        assert_eq!(
            Nevra::parse("foo-1.2.3-1.i386.rpm").unwrap(),
            Nevra {
                name: "foo",
                epoch: None,
                version: "1.2.3",
                release: "1",
                architecture: "i386"
            }
        );
    }

    #[test]
    fn test_nevra_long() {
        assert_eq!(
            Nevra::parse("foo-bar-baz-1.2:3.4.5-6.7.src").unwrap(),
            Nevra {
                name: "foo-bar-baz",
                epoch: Some("1.2"),
                version: "3.4.5",
                release: "6.7",
                architecture: "src"
            }
        );
    }

    #[test]
    fn test_nevra_invalid() {
        assert!(Nevra::parse("foo").is_err()); // no arch+version+release
        assert!(Nevra::parse("foo.i386").is_err()); // no version+release
        assert!(Nevra::parse("foo-1").is_err()); // no arch+release
        assert!(Nevra::parse("foo-1.i386").is_err()); // no release
        assert!(Nevra::parse("foo-1-1").is_err()); // no arch
    }

    #[test]
    fn test_merge_evr() {
        assert_eq!(merge_evr(None, "1.2.3", "0.1"), "1.2.3-0.1".to_string());
        assert_eq!(
            merge_evr(Some("0"), "1.2.3", "0.1"),
            "1.2.3-0.1".to_string()
        );
        assert_eq!(
            merge_evr(Some("1"), "1.2.3", "0.1"),
            "1:1.2.3-0.1".to_string()
        );
    }
}
