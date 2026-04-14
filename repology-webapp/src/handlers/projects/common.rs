// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::{HashMap, HashSet};

use sqlx::FromRow;

use repology_common::{PackageFlags, PackageStatus};

use crate::package::summarization::DisplayVersion;
use crate::package::traits::{PackageWithFlags, PackageWithStatus, PackageWithVersion};

#[derive(Debug, FromRow)]
pub struct ProjectForListing {
    pub effname: String,
    #[sqlx(try_from = "i16")]
    pub num_families: u32,
    pub has_related: bool,
}

#[derive(Debug, FromRow)]
pub struct PackageForListing {
    pub repo: String,
    pub family: String,
    #[expect(unused)]
    pub visiblename: String, // remove if it's not really used anywhere
    pub effname: String,
    pub version: String,
    pub status: PackageStatus,
    pub flags: i32,
    pub maintainers: Vec<String>,
}

impl PackageWithVersion for PackageForListing {
    fn version(&self) -> &str {
        &self.version
    }
}
impl PackageWithFlags for PackageForListing {
    fn flags(&self) -> PackageFlags {
        PackageFlags::from_bits(self.flags as u32).expect("flags must be deserializable")
    }
}
impl PackageWithStatus for PackageForListing {
    fn status(&self) -> PackageStatus {
        self.status
    }
}

type CategorizedDisplayVersionsBucketKey<'a> = (&'a str, PackageStatus, i32);

struct CategorizedDisplayVersionsBucket<'a> {
    pub display_version: DisplayVersion,
    pub families: HashSet<&'a str>,
}

#[derive(Default)]
struct CategorizedDisplayVersionsBuckets<'a> {
    pub focused:
        HashMap<CategorizedDisplayVersionsBucketKey<'a>, CategorizedDisplayVersionsBucket<'a>>,
    pub newest:
        HashMap<CategorizedDisplayVersionsBucketKey<'a>, CategorizedDisplayVersionsBucket<'a>>,
    pub outdated:
        HashMap<CategorizedDisplayVersionsBucketKey<'a>, CategorizedDisplayVersionsBucket<'a>>,
    pub ignored:
        HashMap<CategorizedDisplayVersionsBucketKey<'a>, CategorizedDisplayVersionsBucket<'a>>,
}

#[derive(Default)]
pub struct CategorizedDisplayVersions {
    pub focused: Vec<DisplayVersion>,
    pub newest: Vec<DisplayVersion>,
    pub outdated: Vec<DisplayVersion>,
    pub ignored: Vec<DisplayVersion>,
}

fn finalize_buckets<T>(
    buckets: HashMap<T, CategorizedDisplayVersionsBucket>,
) -> Vec<DisplayVersion> {
    let mut res: Vec<_> = buckets
        .into_values()
        .map(|bucket| {
            bucket
                .display_version
                .with_spread(bucket.families.len().try_into().unwrap_or(1))
        })
        .collect();

    res.sort_by(|a, b| a.cmp(b).reverse());
    res
}

pub fn packages_to_categorized_display_versions_per_project(
    packages: &[PackageForListing],
    selected_repository: Option<&str>,
    selected_maintainer: Option<&str>,
) -> HashMap<String, CategorizedDisplayVersions> {
    let mut by_project: HashMap<&str, CategorizedDisplayVersionsBuckets> = Default::default();

    let want_focus = selected_repository.is_some() || selected_maintainer.is_some();

    for package in packages {
        let project_entry = by_project.entry(&package.effname).or_default();

        let focused = want_focus
            && selected_repository
                .is_none_or(|selected_repository| package.repo == selected_repository)
            && selected_maintainer.is_none_or(|selected_maintainer| {
                package
                    .maintainers
                    .iter()
                    .any(|maintainer| maintainer == selected_maintainer)
            });

        let mut display_version = DisplayVersion::from_package(package);

        let category_entry = {
            use PackageStatus::*;
            match package.status {
                _ if focused => &mut project_entry.focused,
                Outdated | Legacy => {
                    // we don't need to differentiate Legacy here
                    // XXX: move this into DisplayVersion::from_package?
                    display_version.status = Outdated;
                    &mut project_entry.outdated
                }
                Devel | Newest | Unique => &mut project_entry.newest,
                _ => &mut project_entry.ignored,
            }
        };

        let key = (
            package.version.as_ref(),
            display_version.status,
            display_version.metaorder,
        );

        category_entry
            .entry(key)
            .or_insert_with(|| CategorizedDisplayVersionsBucket {
                display_version,
                families: Default::default(),
            })
            .families
            .insert(package.family.as_ref());
    }

    by_project
        .into_iter()
        .map(|(project_name, categorized_buckets)| {
            (
                project_name.into(),
                CategorizedDisplayVersions {
                    focused: finalize_buckets(categorized_buckets.focused),
                    newest: finalize_buckets(categorized_buckets.newest),
                    outdated: finalize_buckets(categorized_buckets.outdated),
                    ignored: finalize_buckets(categorized_buckets.ignored),
                },
            )
        })
        .collect()
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_packages() {
        #[derive(Debug, PartialEq, Eq)]
        struct SimplifiedCategorizedVersions<'a> {
            focused: Vec<&'a str>,
            newest: Vec<&'a str>,
            outdated: Vec<&'a str>,
            ignored: Vec<&'a str>,
        }

        impl<'a> SimplifiedCategorizedVersions<'a> {
            fn from(input: &'a CategorizedDisplayVersions) -> Self {
                Self {
                    focused: input.focused.iter().map(|v| v.version.as_str()).collect(),
                    newest: input.newest.iter().map(|v| v.version.as_str()).collect(),
                    outdated: input.outdated.iter().map(|v| v.version.as_str()).collect(),
                    ignored: input.ignored.iter().map(|v| v.version.as_str()).collect(),
                }
            }
        }

        let gen_package =
            |version: i32, status: PackageStatus, repo: &str, maintainers: &[&str]| {
                PackageForListing {
                    repo: repo.to_string(),
                    family: "".to_string(),
                    visiblename: "".to_string(),
                    effname: "myproject".to_string(),
                    version: format!("{}", version),
                    status,
                    flags: 0,
                    maintainers: maintainers.iter().map(|m| m.to_string()).collect(),
                }
            };

        let packages = vec![
            // duplicated entry, should not be repeated in the output
            gen_package(10, PackageStatus::Newest, "repo_a", &["bar@bar", "foo@foo"]),
            gen_package(10, PackageStatus::Newest, "repo_a", &["bar@bar", "foo@foo"]),
            gen_package(11, PackageStatus::Outdated, "repo_a", &["foo@foo"]),
            gen_package(12, PackageStatus::Incorrect, "repo_a", &["foo@foo"]),
            gen_package(20, PackageStatus::Newest, "repo_a", &["bar@bar"]),
            gen_package(21, PackageStatus::Outdated, "repo_a", &[]),
            gen_package(22, PackageStatus::Incorrect, "repo_a", &[]),
            gen_package(30, PackageStatus::Newest, "repo_b", &["bar@bar", "foo@foo"]),
            gen_package(31, PackageStatus::Outdated, "repo_b", &["foo@foo"]),
            gen_package(32, PackageStatus::Incorrect, "repo_b", &["foo@foo"]),
            gen_package(40, PackageStatus::Newest, "repo_b", &["bar@bar"]),
            gen_package(41, PackageStatus::Outdated, "repo_b", &[]),
            gen_package(42, PackageStatus::Incorrect, "repo_b", &[]),
        ];

        assert_eq!(
            SimplifiedCategorizedVersions::from(
                &packages_to_categorized_display_versions_per_project(&packages, None, None)
                    .remove("myproject")
                    .unwrap()
            ),
            SimplifiedCategorizedVersions {
                focused: vec![],
                newest: vec!["40", "30", "20", "10"],
                outdated: vec!["41", "31", "21", "11"],
                ignored: vec!["42", "32", "22", "12"],
            }
        );

        assert_eq!(
            SimplifiedCategorizedVersions::from(
                &packages_to_categorized_display_versions_per_project(
                    &packages,
                    Some("repo_a"),
                    None
                )
                .remove("myproject")
                .unwrap()
            ),
            SimplifiedCategorizedVersions {
                focused: vec!["22", "21", "20", "12", "11", "10"],
                newest: vec!["40", "30"],
                outdated: vec!["41", "31"],
                ignored: vec!["42", "32"],
            }
        );

        assert_eq!(
            SimplifiedCategorizedVersions::from(
                &packages_to_categorized_display_versions_per_project(
                    &packages,
                    None,
                    Some("foo@foo")
                )
                .remove("myproject")
                .unwrap()
            ),
            SimplifiedCategorizedVersions {
                focused: vec!["32", "31", "30", "12", "11", "10"],
                newest: vec!["40", "20"],
                outdated: vec!["41", "21"],
                ignored: vec!["42", "22"],
            }
        );

        assert_eq!(
            SimplifiedCategorizedVersions::from(
                &packages_to_categorized_display_versions_per_project(
                    &packages,
                    Some("repo_a"),
                    Some("foo@foo")
                )
                .remove("myproject")
                .unwrap()
            ),
            SimplifiedCategorizedVersions {
                focused: vec!["12", "11", "10"],
                newest: vec!["40", "30", "20"],
                outdated: vec!["41", "31", "21"],
                ignored: vec!["42", "32", "22"],
            }
        );
    }
}
