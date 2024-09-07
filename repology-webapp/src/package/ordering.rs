// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use libversion::version_compare;

use repology_common::PackageFlags;

use crate::package::traits::{PackageWithFlags, PackageWithVersion};
use crate::package::version::package_version_flags;

fn package_metaorder<T>(package: &T) -> i32
where
    T: PackageWithVersion + PackageWithFlags,
{
    if package.flags().contains(PackageFlags::Rolling) {
        1
    } else if package.flags().contains(PackageFlags::Outdated) {
        -1
    } else {
        0
    }
}

pub mod by_version_descending {
    use super::*;

    pub fn compare<T>(a: &T, b: &T) -> std::cmp::Ordering
    where
        T: PackageWithVersion + PackageWithFlags,
    {
        package_metaorder(a)
            .cmp(&package_metaorder(b))
            .then_with(|| {
                version_compare(
                    (a.version(), package_version_flags(a)),
                    (b.version(), package_version_flags(b)),
                )
            })
            .reverse()
    }

    #[expect(dead_code)]
    pub fn sort<T>(packages: &mut [T])
    where
        T: PackageWithVersion + PackageWithFlags,
    {
        packages.sort_by(compare);
    }
}
