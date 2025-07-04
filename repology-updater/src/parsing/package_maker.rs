// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::package::Package;
use crate::parsing::error::PackageParsingError;
use crate::parsing::utils::version::VersionStripper;

#[derive(Debug, Clone, Default)]
pub struct PackageMaker {
    projectname_seed: Option<String>,
    version: Option<String>,
}

impl PackageMaker {
    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.projectname_seed = Some(name.into());
        self
    }

    pub fn set_version(&mut self, version: impl Into<String>) -> &mut Self {
        self.version = Some(version.into());
        self
    }

    pub fn set_version_stripped(
        &mut self,
        version: impl Into<String>,
        stripper: &VersionStripper,
    ) -> &mut Self {
        let stripped = stripper.apply(&version.into()).to_string();
        self.version = Some(stripped);
        // TODO: fill origversion
        self
    }

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
