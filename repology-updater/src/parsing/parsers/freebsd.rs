// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Context, anyhow, bail};
use std::io::BufRead as _;
use std::path::Path;

use repology_common::LinkType;

use crate::parsing::package_maker::{NameType, PackageMaker};
use crate::parsing::parser::{PackageParser, PackageSink};
use crate::parsing::utils::maintainers::extract_maintainers;
use crate::parsing::utils::version::VersionStripper;

const EXPECTED_FIELDS_COUNT: usize = 13;
const VERSION_STRIPPER: VersionStripper = VersionStripper::new()
    .with_strip_right(",")
    .with_strip_right("_");

pub struct FreeBsdParser {}

impl FreeBsdParser {
    fn parse_line(line: &str, sink: &mut dyn PackageSink) -> anyhow::Result<()> {
        let fields: Vec<_> = line.trim().split('|').collect();

        if fields.len() != EXPECTED_FIELDS_COUNT {
            bail!(
                "expected {} fields, got {}",
                EXPECTED_FIELDS_COUNT,
                fields.len()
            );
        }

        let mut pkg = PackageMaker::default();

        let (package_name, version) = fields[0]
            .rsplit_once('-')
            .ok_or_else(|| anyhow!("expected <package name>-<version> in the first field"))?;

        let mut path_comps = fields[1].rsplit('/');
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
        pkg.set_summary(fields[3]);
        pkg.add_maintainers(extract_maintainers(&fields[5]));
        pkg.add_categories(fields[6].split_ascii_whitespace());
        pkg.add_links(
            LinkType::UpstreamHomepage,
            fields[9].split_ascii_whitespace(),
        );

        Ok(sink.push(pkg)?)
    }
}

impl PackageParser for FreeBsdParser {
    fn parse(&self, path: &Path, sink: &mut dyn PackageSink) -> anyhow::Result<()> {
        let f = std::fs::File::open_buffered(path)?;

        for (nline, line) in f.lines().enumerate() {
            Self::parse_line(&line?, sink).with_context(|| format!("on line {}", nline + 1))?;
        }

        Ok(())
    }
}
