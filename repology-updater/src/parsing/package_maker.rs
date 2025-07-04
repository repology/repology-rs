// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::package::Package;
use crate::parsing::error::PackageParsingError;

#[derive(Debug, Clone, Default)]
pub struct PackageMaker {
    pub projectname_seed: Option<String>,
    pub version: Option<String>,
}

impl PackageMaker {
    pub fn finalize(self) -> Result<Package, PackageParsingError> {
        let projectname_seed = self
            .projectname_seed
            .ok_or(PackageParsingError::MissingProjectNameSeed)?;
        if projectname_seed.is_empty() {
            return Err(PackageParsingError::EmptyProjectNameSeed);
        }
        let version = self.version.ok_or(PackageParsingError::MissingVersion)?;
        if version.is_empty() {
            return Err(PackageParsingError::EmptyVersion);
        }

        Ok(Package {
            projectname_seed,
            version,
        })
    }
}
