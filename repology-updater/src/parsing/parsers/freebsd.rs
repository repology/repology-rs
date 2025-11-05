// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Context, anyhow};
use std::io::BufRead as _;
use std::path::Path;

use repology_common::LinkType;

use crate::parsing::package_maker::{NameType, PackageMaker};
use crate::parsing::parser::{PackageParser, PackageProcessor};
use crate::parsing::utils::maintainers::extract_maintainers;
use crate::parsing::utils::version::VersionStripper;

#[allow(unused)]
mod data {
    use anyhow::bail;

    pub struct Package<'a> {
        pub pkgname: &'a str,
        pub path: &'a str,
        pub prefix: &'a str,
        pub comment: &'a str,
        pub descr: &'a str,
        pub maintainer: &'a str,
        pub categories: &'a str,
        pub extract_depends: &'a str,
        pub patch_depends: &'a str,
        pub fetch_depends: &'a str,
        pub build_depends: &'a str,
        pub run_depends: &'a str,
        pub www: &'a str,
    }

    impl<'a> TryFrom<&'a str> for Package<'a> {
        type Error = anyhow::Error;

        fn try_from(line: &'a str) -> anyhow::Result<Package<'a>> {
            const EXPECTED_FIELDS_COUNT: usize = 13;
            let fields: Vec<_> = line.trim().split('|').collect();
            if fields.len() != EXPECTED_FIELDS_COUNT {
                bail!(
                    "expected {} fields, got {}",
                    EXPECTED_FIELDS_COUNT,
                    fields.len()
                );
            };
            Ok(Self {
                pkgname: fields[0],
                path: fields[1],
                prefix: fields[2],
                comment: fields[3],
                descr: fields[4],
                maintainer: fields[5],
                categories: fields[6],
                build_depends: fields[7],
                run_depends: fields[8],
                www: fields[9],
                extract_depends: fields[10],
                patch_depends: fields[11],
                fetch_depends: fields[12],
            })
        }
    }
}

const VERSION_STRIPPER: VersionStripper = VersionStripper::new()
    .with_strip_right(",")
    .with_strip_right("_");

pub struct FreeBsdParser {}

impl FreeBsdParser {
    fn parse_line(line: &str, process: &mut dyn PackageProcessor) -> anyhow::Result<()> {
        let pkgdata = data::Package::try_from(line)?;

        let mut pkg = PackageMaker::default();

        let (package_name, version) = pkgdata
            .pkgname
            .rsplit_once('-')
            .ok_or_else(|| anyhow!("expected <package name>-<version> in the first field"))?;

        let mut path_comps = pkgdata.path.rsplit('/');
        let port_name = path_comps
            .next()
            .expect("split always returns at least one component");
        let port_category = path_comps.next().ok_or_else(|| {
            anyhow!("unexpectedly short port path (expected at least category/name)")
        })?;

        pkg.set_names(package_name, NameType::BinName | NameType::ProjectNameSeed);
        pkg.set_names(
            format!("{port_category}/{port_name}"),
            NameType::SrcName | NameType::DisplayName | NameType::TrackName,
        );
        pkg.set_version_stripped(version, &VERSION_STRIPPER);
        pkg.set_summary(pkgdata.comment);
        pkg.add_maintainers(extract_maintainers(&pkgdata.maintainer));
        pkg.add_categories(pkgdata.categories.split_ascii_whitespace());
        pkg.add_links(
            LinkType::UpstreamHomepage,
            pkgdata.www.split_ascii_whitespace(),
        );

        Ok(process(pkg)?)
    }
}

impl PackageParser for FreeBsdParser {
    fn parse(&self, path: &Path, process: &mut dyn PackageProcessor) -> anyhow::Result<()> {
        let f = std::fs::File::open_buffered(path)?;

        for (nline, line) in f.lines().enumerate() {
            Self::parse_line(&line?, process).with_context(|| format!("on line {}", nline + 1))?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    parser_test!(FreeBsdParser {}, freebsd, ok);
    parser_test!(FreeBsdParser {}, freebsd, error_missing_field);
    parser_test!(FreeBsdParser {}, freebsd, error_extra_field);
    parser_test!(FreeBsdParser {}, freebsd, error_missing_version);
    parser_test!(FreeBsdParser {}, freebsd, error_missing_name);
    parser_test!(FreeBsdParser {}, freebsd, error_bad_package_format);
    parser_test!(FreeBsdParser {}, freebsd, error_bad_package_path);
}
