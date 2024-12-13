// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use repology_common::{PackageFlags, PackageStatus};

use crate::package::ordering::package_metaorder;
use crate::package::traits::{PackageWithFlags, PackageWithStatus, PackageWithVersion};
use crate::package::version::package_version_flags;

#[derive(Debug)]
pub struct DisplayVersion {
    // TODO: switch from version + versionflags to libversion::Version
    pub version: String,
    pub status: PackageStatus,
    pub metaorder: i32,
    pub versionflags: libversion::Flags,
    pub vulnerable: bool,
    pub recalled: bool,
    pub spread: u32,
}

impl DisplayVersion {
    pub fn from_package<T>(package: &T) -> Self
    where
        T: PackageWithVersion + PackageWithFlags + PackageWithStatus,
    {
        Self {
            version: package.version().into(),
            status: package.status(),
            metaorder: package_metaorder(package),
            versionflags: package_version_flags(package),
            vulnerable: package.flags().contains(PackageFlags::Vulnerable),
            recalled: package.flags().contains(PackageFlags::Recalled),
            spread: 1,
        }
    }

    pub fn with_spread(mut self, spread: u32) -> Self {
        self.spread = spread;
        self
    }
}

impl PartialEq<Self> for DisplayVersion {
    fn eq(&self, other: &Self) -> bool {
        libversion::version_compare(
            (&self.version, self.versionflags),
            (&other.version, other.versionflags),
        ) == std::cmp::Ordering::Equal
            && self.status == other.status
            && self.metaorder == other.metaorder
            && self.vulnerable == other.vulnerable
            && self.recalled == other.recalled
            && self.spread == other.spread
    }
}

impl PartialOrd for DisplayVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DisplayVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.metaorder
            .cmp(&other.metaorder)
            .then_with(|| {
                libversion::version_compare(
                    (&self.version, self.versionflags),
                    (&other.version, other.versionflags),
                )
            })
            .then_with(|| self.status.cmp(&other.status))
            .then_with(|| self.vulnerable.cmp(&other.vulnerable).reverse())
            .then_with(|| self.recalled.cmp(&other.recalled).reverse())
            .then_with(|| self.spread.cmp(&other.spread))
            .then_with(|| self.version.cmp(&other.version))
    }
}

impl Eq for DisplayVersion {}
