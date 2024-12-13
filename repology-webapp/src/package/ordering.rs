// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use libversion::version_compare;

use repology_common::PackageFlags;

use crate::package::traits::{PackageWithDisplayName, PackageWithFlags, PackageWithVersion};
use crate::package::version::package_version_flags;

pub fn package_metaorder<T>(package: &T) -> i32
where
    T: PackageWithVersion + PackageWithFlags,
{
    if package.flags().contains(PackageFlags::Rolling) {
        1
    } else if package.flags().contains(PackageFlags::Sink) {
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

    pub fn sort<T>(packages: &mut [T])
    where
        T: PackageWithVersion + PackageWithFlags,
    {
        packages.sort_by(compare);
    }
}

pub mod by_name_asc_version_desc {
    use super::*;

    pub fn compare<T>(a: &T, b: &T) -> std::cmp::Ordering
    where
        T: PackageWithDisplayName + PackageWithVersion + PackageWithFlags,
    {
        a.display_name()
            .cmp(b.display_name())
            .then_with(|| package_metaorder(a).cmp(&package_metaorder(b)).reverse())
            .then_with(|| {
                version_compare(
                    (a.version(), package_version_flags(a)),
                    (b.version(), package_version_flags(b)),
                )
                .reverse()
            })
    }

    pub fn sort<T>(packages: &mut [T])
    where
        T: PackageWithDisplayName + PackageWithVersion + PackageWithFlags,
    {
        packages.sort_by(compare);
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Package {
        version: &'static str,
        flags: PackageFlags,
    }

    impl PackageWithVersion for Package {
        fn version(&self) -> &str {
            self.version
        }
    }

    impl PackageWithFlags for Package {
        fn flags(&self) -> PackageFlags {
            self.flags
        }
    }

    #[test]
    fn test_sorting() {
        let expected = vec![
            Package {
                version: "2.0",
                flags: PackageFlags::Rolling,
            },
            Package {
                version: "1.0p1",
                flags: PackageFlags::Rolling | PackageFlags::PIsPatch,
            },
            Package {
                version: "1.0",
                flags: PackageFlags::Rolling,
            },
            Package {
                version: "1.0p1",
                flags: PackageFlags::Rolling,
            },
            Package {
                version: "2.0",
                flags: Default::default(),
            },
            Package {
                version: "1.0p1",
                flags: PackageFlags::PIsPatch,
            },
            Package {
                version: "1.0",
                flags: Default::default(),
            },
            Package {
                version: "1.0p1",
                flags: Default::default(),
            },
            Package {
                version: "2.0",
                flags: PackageFlags::Sink,
            },
            Package {
                version: "1.0p1",
                flags: PackageFlags::Sink | PackageFlags::PIsPatch,
            },
            Package {
                version: "1.0",
                flags: PackageFlags::Sink,
            },
            Package {
                version: "1.0p1",
                flags: PackageFlags::Sink,
            },
        ];

        let mut packages = expected.clone();
        packages.reverse();
        by_version_descending::sort(&mut packages);
        assert_eq!(packages, expected);
    }
}
