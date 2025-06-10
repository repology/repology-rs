// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use libversion::{Flags as VersionFlags, VersionStr};

use repology_common::PackageFlags;

use crate::package::traits::{PackageWithFlags, PackageWithVersion};

pub fn package_version_flags<T>(package: &T) -> VersionFlags
where
    T: PackageWithVersion + PackageWithFlags,
{
    let mut flags = VersionFlags::empty();
    flags.set(
        VersionFlags::PIsPatch,
        package.flags().contains(PackageFlags::PIsPatch),
    );
    flags.set(
        VersionFlags::AnyIsPatch,
        package.flags().contains(PackageFlags::AnyIsPatch),
    );
    flags
}

pub fn package_version<T>(package: &T) -> VersionStr<'_>
where
    T: PackageWithVersion + PackageWithFlags,
{
    VersionStr::new(package.version(), package_version_flags(package))
}
