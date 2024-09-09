// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use repology_common::{PackageFlags, PackageStatus};

use crate::package::ordering::by_version_descending;
use crate::package::traits::{
    PackageWithFlags, PackageWithRepositoryName, PackageWithStatus, PackageWithVersion,
};

fn is_representative_package<T>(package: &T, allow_ignored: bool) -> bool
where
    T: PackageWithVersion + PackageWithFlags + PackageWithStatus,
{
    !package.flags().contains(PackageFlags::Recalled)
        && (allow_ignored
            || !matches!(
                package.status(),
                PackageStatus::Ignored
                    | PackageStatus::Incorrect
                    | PackageStatus::Untrusted
                    | PackageStatus::NoScheme
                    | PackageStatus::Rolling
            ))
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

struct RepresentativePackageChooser<'a, T> {
    fallback: Option<&'a T>,
    representative: Option<&'a T>,
}

impl<'a, T> Default for RepresentativePackageChooser<'a, T> {
    fn default() -> Self {
        Self {
            fallback: None,
            representative: None,
        }
    }
}

impl<'a, T> RepresentativePackageChooser<'a, T>
where
    T: PackageWithVersion + PackageWithFlags + PackageWithStatus,
{
    pub fn push(&mut self, package: &'a T, allow_ignored: bool) {
        if is_representative_package(package, allow_ignored) {
            update_optional_max(&mut self.representative, package);
        }

        update_optional_max(&mut self.fallback, package);
    }

    pub fn get(&self) -> Option<&'a T> {
        self.representative.or(self.fallback)
    }
}

pub fn pick_representative_package<T>(packages: &[T], allow_ignored: bool) -> Option<&T>
where
    T: PackageWithVersion + PackageWithFlags + PackageWithStatus,
{
    let mut chooser: RepresentativePackageChooser<T> = Default::default();
    packages
        .iter()
        .for_each(|package| chooser.push(package, allow_ignored));
    chooser.get()
}

pub fn pick_representative_package_per_repository<T>(
    packages: &[T],
    allow_ignored: bool,
) -> HashMap<String, &T>
where
    T: PackageWithVersion + PackageWithFlags + PackageWithStatus + PackageWithRepositoryName,
{
    let mut choosers: HashMap<&str, RepresentativePackageChooser<T>> = Default::default();
    packages.iter().for_each(|package| {
        choosers
            .entry(package.repository_name())
            .or_default()
            .push(package, allow_ignored)
    });
    choosers
        .into_iter()
        .map(|(repository_name, chooser)| {
            (
                String::from(repository_name),
                chooser
                    .get()
                    .expect("per-repository chooser is expected to contain at least one package"),
            )
        })
        .collect()
}

#[cfg(test)]
#[coverage(off)]
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
