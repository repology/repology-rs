// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::package::summarization::DisplayVersion;
use crate::repository_data::RepositoriesDataSnapshot;

use super::Link;
use super::emails::MaintainerEmailsAggregator;
use super::slices::*;

type StringSliceAccumulator<'a> = HashMap<&'a str, HashSet<&'a str>>;
type LinkSliceAccumulator<'a> = HashMap<(i32, Option<&'a str>), HashSet<&'a str>>;

#[derive(Default)]
pub struct SlicesAccumulator<'a> {
    // XXX: can use non-owninig DisplayVersion here
    pub versions: Vec<(DisplayVersion, &'a str)>,
    pub repositories: HashSet<&'a str>,
    pub string_slices: HashMap<StringSliceType, StringSliceAccumulator<'a>>,
    pub link_slices: HashMap<LinkSliceType, LinkSliceAccumulator<'a>>,
    pub maintainer_emails: MaintainerEmailsAggregator<'a>,
}

impl<'a> SlicesAccumulator<'a> {
    pub fn add_string_slice(
        &mut self,
        slice_type: StringSliceType,
        value: &'a str,
        family: &'a str,
    ) {
        self.string_slices
            .entry(slice_type)
            .or_default()
            .entry(value)
            .or_default()
            .insert(family);
    }

    pub fn add_link_slice(
        &mut self,
        slice_type: LinkSliceType,
        link_id: i32,
        fragment: Option<&'a str>,
        family: &'a str,
    ) {
        self.link_slices
            .entry(slice_type)
            .or_default()
            .entry((link_id, fragment))
            .or_default()
            .insert(family);
    }

    pub fn get_all_link_ids(&self) -> Vec<i32> {
        let mut ids: HashSet<i32> = Default::default();
        self.link_slices.values().for_each(|map| {
            map.keys().for_each(|(link_id, _)| {
                ids.insert(*link_id);
            })
        });
        ids.into_iter().collect()
    }

    pub fn finalize(
        self,
        links: &'a HashMap<i32, Link>,
        repositories_data: &'a RepositoriesDataSnapshot,
    ) -> Slices<'a> {
        let versions = {
            // XXX: this code is somewhat similar to packages_to_categorized_display_versions_per_project(), merge code?
            let mut current_version: Option<DisplayVersion> = None;
            let mut current_families: HashSet<&str> = Default::default();
            let mut res: Vec<DisplayVersion> = Default::default();

            for (version, family) in self
                .versions
                .into_iter()
                .sorted_by(|a, b| a.cmp(b).reverse())
            {
                if current_version.as_ref().is_some_and(|current| {
                    version.version == current.version
                        && version.status == current.status
                        && version.vulnerable == current.vulnerable
                }) {
                    // same key
                    current_families.insert(family);
                } else {
                    if let Some(res_version) = current_version.take() {
                        res.push(
                            res_version
                                .with_spread(std::mem::take(&mut current_families).len() as u32),
                        );
                    }
                    current_version = Some(version);
                    current_families.insert(family);
                }
            }

            if let Some(res_version) = current_version {
                res.push(res_version.with_spread(current_families.len() as u32));
            }

            // additional sort which should take changed spreads into account
            res.sort_by(|a, b| a.cmp(b).reverse());
            res
        };

        Slices {
            versions,
            repositories: repositories_data
                .active_repositories()
                .filter(|repository_data| self.repositories.contains(repository_data.name.as_str()))
                .collect(),
            string_slices: self
                .string_slices
                .into_iter()
                .map(|(slice_type, slice)| {
                    (
                        slice_type,
                        slice
                            .into_iter()
                            .map(|(value, families)| StringSliceItem {
                                value,
                                spread: families.len(),
                            })
                            .sorted_by(|a, b| a.value.cmp(b.value))
                            .collect(),
                    )
                })
                .collect(),
            link_slices: self
                .link_slices
                .into_iter()
                .map(|(slice_type, slice)| {
                    (
                        slice_type,
                        slice
                            .into_iter()
                            .map(|((link_id, fragment), families)| {
                                let link = links
                                    .get(&link_id)
                                    .expect("link referenced from package must exist");
                                let url = if let Some(fragment) = fragment {
                                    Cow::Owned(link.url.clone() + "#" + fragment)
                                } else {
                                    Cow::Borrowed(link.url.as_str())
                                };
                                LinkSliceItem {
                                    url,
                                    spread: families.len(),
                                    link,
                                }
                            })
                            .sorted_by(|a, b| a.url.cmp(&b.url))
                            .collect(),
                    )
                })
                .collect(),
            maintainer_emails: self.maintainer_emails.into_joined_addresses(),
        }
    }
}
