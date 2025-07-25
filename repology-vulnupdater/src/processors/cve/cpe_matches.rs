// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use metrics::counter;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::str::FromStr as _;

use crate::cpe::Cpe;

use super::schema;

#[derive(PartialEq, Eq, Hash, Debug)]
struct VersionBound<'a> {
    version: Cow<'a, str>,
    excluded: bool,
}

impl<'a> VersionBound<'a> {
    pub fn from_either(
        including: Option<Cow<'a, str>>,
        excluding: Option<Cow<'a, str>>,
    ) -> Option<Self> {
        let excluded = including.is_none();
        including
            .or(excluding)
            .map(|version| Self { version, excluded })
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct VersionRange<'a> {
    start: Option<VersionBound<'a>>,
    end: Option<VersionBound<'a>>,
}

impl<'a> VersionRange<'a> {
    pub fn from_single_version(version: Cow<'a, str>) -> Self {
        Self {
            start: Some(VersionBound {
                version: version.clone(),
                excluded: false,
            }),
            end: Some(VersionBound {
                version,
                excluded: false,
            }),
        }
    }
}

#[derive(Default)]
pub struct CpeMatches<'a> {
    matches: HashMap<Cpe, HashSet<VersionRange<'a>>>,
}

const OS_VENDOR_PRODUCT_WHITELIST: &[(&str, &str)] = &[
    ("coreboot", "coreboot"),
    ("linux", "linux"),
    ("linux", "linux_kernel"),
    ("xen", "xen"),
];

fn is_good_cpe(cpe: &Cpe) -> bool {
    match cpe.part {
        crate::cpe::Part::Applications => true,
        crate::cpe::Part::OperatingSystems => OS_VENDOR_PRODUCT_WHITELIST
            .iter()
            .any(|(wl_vendor, wl_product)| cpe.vendor == *wl_vendor && cpe.product == *wl_product),
        _ => false,
    }
}

impl<'a> CpeMatches<'a> {
    fn collect_from_node(&mut self, node: &'a schema::Node<'a>) {
        if node.operator != "OR" {
            // TODO: complex node trees are not supported yet
            counter!("repology_vulnupdater_processor_cve_nodes_total", "status" => "skipped", "skip_reason" => "operator other than OR").increment(1);
            return;
        }

        if node.negate {
            // TODO: complex node trees are not supported yet
            counter!("repology_vulnupdater_processor_cve_nodes_total", "status" => "skipped", "skip_reason" => "negate").increment(1);
            return;
        }

        counter!("repology_vulnupdater_processor_cve_nodes_total", "status" => "processed")
            .increment(1);

        for cpe_match in &node.cpe_match {
            counter!("repology_vulnupdater_processor_cve_cpes_total").increment(1);

            if !cpe_match.vulnerable {
                // TODO: investigate if vulnerability exclusions are relevant
                counter!("repology_vulnupdater_processor_cve_cpes_total", "status" => "skipped", "skip_reason" => "not vulnerable").increment(1);
                continue;
            }

            let Ok(cpe) = Cpe::from_str(&cpe_match.criteria) else {
                // TODO: log or fix these cases
                counter!("repology_vulnupdater_processor_cve_cpes_total", "status" => "skipped", "skip_reason" => "unparsable CPE").increment(1);
                continue;
            };

            if !is_good_cpe(&cpe) {
                // TODO: recheck if we need os matches
                counter!("repology_vulnupdater_processor_cve_cpes_total", "status" => "skipped", "skip_reason" => "uninteresting CPE").increment(1);
                continue;
            }

            if cpe.version == "-" {
                counter!("repology_vulnupdater_processor_cve_cpes_total", "status" => "skipped", "skip_reason" => "version is -").increment(1);
                continue;
            }

            if cpe.version == "*" {
                // version defined by ranges
                let start = VersionBound::from_either(
                    cpe_match.version_start_including.clone(),
                    cpe_match.version_start_excluding.clone(),
                );
                let end = VersionBound::from_either(
                    cpe_match.version_end_including.clone(),
                    cpe_match.version_end_excluding.clone(),
                );

                counter!("repology_vulnupdater_processor_cve_cpes_total", "status" => "accepted", "type" => "range").increment(1);
                self.matches
                    .entry(cpe)
                    .or_default()
                    .insert(VersionRange { start, end });
            } else {
                let version = if cpe.update != "*" && cpe.update != "-" && !cpe.update.is_empty() {
                    if cpe.update.starts_with(|c: char| c.is_ascii_digit()) {
                        format!("{}-{}", cpe.version, cpe.update)
                    } else {
                        format!("{}{}", cpe.version, cpe.update)
                    }
                } else {
                    cpe.version.clone()
                };

                counter!("repology_vulnupdater_processor_cve_cpes_total", "status" => "accepted", "type" => "single version").increment(1);
                self.matches
                    .entry(cpe)
                    .or_default()
                    .insert(VersionRange::from_single_version(Cow::from(version)));

                // XXX: log if any of version_{start,end}_{including,excluding} is also
                // defined here
            }
        }
    }

    pub fn from_cve(cve: &'a schema::Cve<'a>) -> Self {
        let mut res: Self = Default::default();
        for configuration in &cve.configurations {
            if configuration
                .operator
                .is_some_and(|operator| operator == "AND")
            {
                // TODO: complex node trees are not supported yet
                counter!("repology_vulnupdater_processor_cve_configurations_total", "status" => "skipped", "skip_reason" => "AND operator").increment(1);
                continue;
            }
            counter!("repology_vulnupdater_processor_cve_configurations_total", "status" => "processed").increment(1);

            for node in &configuration.nodes {
                res.collect_from_node(node)
            }
        }
        res
    }

    pub fn vendor_product_pairs_for_sql(&self) -> Vec<String> {
        self.matches
            .keys()
            .map(|cpe| format!("{}:{}", cpe.vendor, cpe.product))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn into_matches_for_sql(self) -> serde_json::Value {
        self.matches
            .into_iter()
            .flat_map(|(cpe, ranges)| ranges.into_iter().map(move |range| (cpe.clone(), range)))
            .map(|(cpe, range)| -> serde_json::Value {
                let start_excluded = range.start.as_ref().is_some_and(|bound| bound.excluded);
                let end_excluded = range.end.as_ref().is_some_and(|bound| bound.excluded);

                [
                    cpe.vendor.into(),
                    cpe.product.into(),
                    cpe.edition.into(),
                    cpe.lang.into(),
                    cpe.sw_edition.into(),
                    cpe.target_sw.into(),
                    cpe.target_hw.into(),
                    cpe.other.into(),
                    range
                        .start
                        .map(|bound| bound.version.into())
                        .unwrap_or(serde_json::Value::Null),
                    range
                        .end
                        .map(|bound| bound.version.into())
                        .unwrap_or(serde_json::Value::Null),
                    start_excluded.into(),
                    end_excluded.into(),
                ]
                .as_slice()
                .into()
            })
            .collect()
    }
}
