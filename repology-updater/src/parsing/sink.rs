// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::parsing::error::PackageParsingError;
use crate::parsing::package_maker::PackageMaker;

pub trait PackageSink {
    fn push(&mut self, package_maker: PackageMaker) -> Result<(), PackageParsingError>;
}
