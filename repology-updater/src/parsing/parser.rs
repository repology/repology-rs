// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use crate::package::ParsedPackage;
use crate::parsing::error::PackageParsingError;
use crate::parsing::package_maker::PackageMaker;

pub trait PackageSink {
    fn push(&mut self, package_maker: PackageMaker) -> Result<(), PackageParsingError>;
}

pub trait PackageParser {
    fn parse(&self, path: &Path, sink: &mut dyn PackageSink) -> anyhow::Result<()>;
}

#[derive(Default)]
pub struct PackageAccumulator {
    pub packages: Vec<ParsedPackage>,
}

impl PackageSink for PackageAccumulator {
    fn push(&mut self, package_maker: PackageMaker) -> Result<(), PackageParsingError> {
        self.packages.push(package_maker.finalize()?);
        Ok(())
    }
}
