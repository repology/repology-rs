// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use repology_common::{PackageFlags, PackageStatus};

use crate::package::ordering::by_version_descending;
use crate::package::traits::{PackageWithFlags, PackageWithStatus, PackageWithVersion};

fn is_representative_package<T>(package: &T, allow_ignored: bool) -> bool
where
    T: PackageWithVersion + PackageWithFlags + PackageWithStatus,
{
    !package.flags().contains(PackageFlags::Recalled)
        && (allow_ignored
            || match package.status() {
                PackageStatus::Ignored
                | PackageStatus::Incorrect
                | PackageStatus::Untrusted
                | PackageStatus::NoScheme
                | PackageStatus::Rolling => false,
                _ => true,
            })
}

fn update_optional_max<'a, T>(target: &mut Option<&'a T>, next: &'a T)
where
    T: PackageWithVersion + PackageWithFlags + PackageWithStatus,
{
    if let Some(old) = *target {
        if by_version_descending::compare(old, next) != std::cmp::Ordering::Greater {
            return;
        }
    }
    *target = Some(next);
}

pub fn pick_representative_package<T>(packages: &[T], allow_ignored: bool) -> Option<&T>
where
    T: PackageWithVersion + PackageWithFlags + PackageWithStatus,
{
    let mut fallback: Option<&T> = None;
    let mut representative: Option<&T> = None;

    packages.iter().for_each(|package| {
        if is_representative_package(package, allow_ignored) {
            update_optional_max(&mut representative, package);
        }

        update_optional_max(&mut fallback, package);
    });

    representative.or(fallback)
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Package {
        version: &'static str,
        flags: PackageFlags,
        status: PackageStatus,
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
    impl PackageWithStatus for Package {
        fn status(&self) -> PackageStatus {
            self.status
        }
    }

    use PackageStatus::*;

    #[test]
    fn representative_empty() {
        let packages: Vec<Package> = vec![];

        assert_eq!(pick_representative_package(&packages, false), None);
    }

    #[test]
    fn representative_max_version() {
        let packages = vec![
            Package {
                version: "2",
                flags: PackageFlags::empty(),
                status: Newest,
            },
            Package {
                version: "3",
                flags: PackageFlags::empty(),
                status: Newest,
            },
            Package {
                version: "1",
                flags: PackageFlags::empty(),
                status: Newest,
            },
        ];

        assert_eq!(
            pick_representative_package(&packages, false),
            Some(&packages[1])
        );
    }

    #[test]
    fn representative_avoid_recalled() {
        let packages = vec![
            Package {
                version: "2",
                flags: PackageFlags::empty(),
                status: Newest,
            },
            Package {
                version: "3",
                flags: PackageFlags::Recalled,
                status: Newest,
            },
            Package {
                version: "1",
                flags: PackageFlags::empty(),
                status: Newest,
            },
        ];

        assert_eq!(
            pick_representative_package(&packages, false),
            Some(&packages[0])
        );
    }

    #[test]
    fn representative_avoid_recalled_fallback() {
        let packages = vec![
            Package {
                version: "2",
                flags: PackageFlags::Recalled,
                status: Newest,
            },
            Package {
                version: "3",
                flags: PackageFlags::Recalled,
                status: Newest,
            },
            Package {
                version: "1",
                flags: PackageFlags::Recalled,
                status: Newest,
            },
        ];

        assert_eq!(
            pick_representative_package(&packages, false),
            Some(&packages[1])
        );
    }

    #[test]
    fn representative_avoid_ignored() {
        let packages = vec![
            Package {
                version: "2",
                flags: PackageFlags::empty(),
                status: Newest,
            },
            Package {
                version: "3",
                flags: PackageFlags::empty(),
                status: Ignored,
            },
            Package {
                version: "1",
                flags: PackageFlags::empty(),
                status: Newest,
            },
        ];

        assert_eq!(
            pick_representative_package(&packages, false),
            Some(&packages[0])
        );
    }

    #[test]
    fn representative_avoid_ignored_not() {
        let packages = vec![
            Package {
                version: "2",
                flags: PackageFlags::empty(),
                status: Newest,
            },
            Package {
                version: "3",
                flags: PackageFlags::empty(),
                status: Ignored,
            },
            Package {
                version: "1",
                flags: PackageFlags::empty(),
                status: Newest,
            },
        ];

        assert_eq!(
            pick_representative_package(&packages, true),
            Some(&packages[1])
        );
    }
}
