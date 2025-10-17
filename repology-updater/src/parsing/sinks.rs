// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::package::ParsedPackage;
use crate::parsing::error::PackageParsingError;
use crate::parsing::package_maker::PackageMaker;
use crate::parsing::sink::PackageSink;

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

#[derive(Default)]
pub struct PackageNullSink {}

impl PackageSink for PackageNullSink {
    fn push(&mut self, _: PackageMaker) -> Result<(), PackageParsingError> {
        Ok(())
    }
}

pub struct PackageCounter<'n, N: ?Sized> {
    pub count: usize,
    next: &'n mut N,
}

impl<'n, N> PackageCounter<'n, N>
where
    N: PackageSink + ?Sized,
{
    pub fn new(next: &'n mut N) -> Self {
        Self { count: 0, next }
    }
}

impl<'n, N> PackageSink for PackageCounter<'n, N>
where
    N: PackageSink + ?Sized,
{
    fn push(&mut self, package_maker: PackageMaker) -> Result<(), PackageParsingError> {
        self.count += 1;
        self.next.push(package_maker)
    }
}

#[derive(Default)]
pub struct PackageDumper {}

impl PackageSink for PackageDumper {
    fn push(&mut self, package_maker: PackageMaker) -> Result<(), PackageParsingError> {
        println!("{:#?}", package_maker.finalize()?);
        Ok(())
    }
}
