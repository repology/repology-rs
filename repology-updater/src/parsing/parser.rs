// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use crate::parsing::error::PackageParsingError;
use crate::parsing::package_maker::PackageMaker;

pub trait PackageProcessor = FnMut(PackageMaker) -> Result<(), PackageParsingError>;

pub trait PackageParser {
    fn parse(&self, path: &Path, sink: &mut dyn PackageProcessor) -> anyhow::Result<()>;
}
