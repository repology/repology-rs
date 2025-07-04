// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Context, anyhow, bail};
use std::io::BufRead as _;
use std::path::Path;

use crate::package::Package;
use crate::parsing::error::PackageParsingError;
use crate::parsing::package_maker::PackageMaker;
use crate::parsing::parser::{PackageParser, PackageSink};

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

        pkg.projectname_seed = Some(name.to_owned());
        pkg.version = Some(version.to_owned());

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

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    use std::io::Write as _;

    use crate::parsing::parser::PackageAccumulator;

    #[test]
    fn test_parse_ok() {
        let dir = tempdir().unwrap();
        let index_path = dir.path().join("INDEX");
        writeln!(
            std::fs::File::create(&index_path).unwrap(),
            "trojka-1.0_2|/usr/ports/games/trojka|/usr/local|Game of skill|/usr/ports/games/trojka/pkg-descr|e@ik.nu|games||||||"
        ).unwrap();

        let mut packages = PackageAccumulator::default();
        FreeBsdParser {}.parse(&index_path, &mut packages).unwrap();

        assert_eq!(
            packages.packages[0],
            Package {
                projectname_seed: "trojka".to_owned(),
                version: "1.0_2".to_owned()
            }
        );
    }

    #[test]
    fn test_parse_fail() {
        let cases = vec![
            // missing field
            "trojka-1.0_2|/usr/ports/games/trojka|/usr/local|Game of skill|/usr/ports/games/trojka/pkg-descr|e@ik.nu|games|||||",
            // extra field
            "trojka-1.0_2|/usr/ports/games/trojka|/usr/local|Game of skill|/usr/ports/games/trojka/pkg-descr|e@ik.nu|games|||||||",
            // bad package name format
            "trojka1.0_2|/usr/ports/games/trojka|/usr/local|Game of skill|/usr/ports/games/trojka/pkg-descr|e@ik.nu|games||||||",
            // no version
            "trojka-|/usr/ports/games/trojka|/usr/local|Game of skill|/usr/ports/games/trojka/pkg-descr|e@ik.nu|games||||||",
            // no name
            "-1.0_2|/usr/ports/games/trojka|/usr/local|Game of skill|/usr/ports/games/trojka/pkg-descr|e@ik.nu|games||||||",
        ];

        for case in cases {
            let dir = tempdir().unwrap();
            let index_path = dir.path().join("INDEX");
            writeln!(std::fs::File::create(&index_path).unwrap(), "{case}",).unwrap();

            let mut _packages = PackageAccumulator::default();
            let res = FreeBsdParser {}.parse(&index_path, &mut _packages);
            assert!(res.is_err());
            eprintln!("{:#?}", res);
        }
    }
}
