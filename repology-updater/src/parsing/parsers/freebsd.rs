// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Context, anyhow, bail};
use std::io::BufRead as _;
use std::path::Path;

use crate::package::Package;
use crate::parsing::error::PackageParsingError;
use crate::parsing::package_maker::PackageMaker;
use crate::parsing::parser::{PackageParser, PackageSink};
use crate::parsing::utils::version::VersionStripper;

const VERSION_STRIPPER: VersionStripper = VersionStripper::new()
    .with_strip_right(",")
    .with_strip_right("_");

pub struct FreeBsdParser {}

impl FreeBsdParser {
    fn parse_line(line: &str, sink: &mut dyn PackageSink) -> anyhow::Result<()> {
        let fields: Vec<_> = line.trim().split('|').collect();

        const EXPECTED_FIELDS_COUNT: usize = 13;

        if fields.len() != EXPECTED_FIELDS_COUNT {
            bail!(
                "expected {} fields, got {}",
                EXPECTED_FIELDS_COUNT,
                fields.len()
            );
        }

        let mut pkg = PackageMaker::default();

        let (name, version) = fields[0]
            .rsplit_once('-')
            .ok_or_else(|| anyhow!("expected <package name>-<version> in the first field"))?;

        pkg.set_name(name);
        pkg.set_version_stripped(version, &VERSION_STRIPPER);

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
