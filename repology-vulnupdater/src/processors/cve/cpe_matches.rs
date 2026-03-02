// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use metrics::counter;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::str::FromStr as _;

use tracing::{info, warn};

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

/// Check if CPE type is acceptable.
///
/// Since Repology operates on packaged applications, we're only
/// inserested in Application type CPEs. However, there are a few
/// exceptions of OperatingSystems type CPEs which can be packaged,
/// for instance Linux kernel.
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
    /// Collects matches from a Node.
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
                counter!("repology_vulnupdater_processor_cve_cpes_total", "status" => "skipped", "skip_reason" => "unparsable CPE").increment(1);
                warn!(
                    criteria = cpe_match.criteria.as_ref(),
                    "rejecting CpeMatch with unparsable CPE"
                );
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

    /// Collects matches from a Configuration.
    ///
    /// Configuration/Node trees may imply complex logic we don't know
    /// how to properly handle yet. This function checks for known cases,
    /// which are currently:
    ///
    /// - Configuration without operator or with OR operatior, which is
    ///   considered trivial (CVE applies to a set of products of any kind).
    /// - Configuration with AND operator and two nodes, one of which
    ///   only lists application matches, and another lists OperatingSystems
    ///   matches. This is supposed to denote an Application only vulnerable
    ///   on specified OSes, so we process application nodes, and ignore OS
    ///   nodes in this case.
    ///
    /// More cases may be added after careful consideration.
    fn collect_from_configuration(&mut self, configuration: &'a schema::Configuration) {
        if configuration.operator == Some("AND") {
            let mut application_nodes = vec![];
            let mut operating_system_nodes = vec![];

            for node in &configuration.nodes {
                let mut node_part = None;

                for cpe_match in &node.cpe_match {
                    let Ok(cpe) = Cpe::from_str(&cpe_match.criteria) else {
                        counter!("repology_vulnupdater_processor_cve_configurations_total", "status" => "skipped", "skip_reason" => "unparsable CPE").increment(1);
                        warn!(
                            criteria = cpe_match.criteria.as_ref(),
                            "rejecting configuration with unparsable CPE"
                        );
                        return;
                    };

                    match node_part {
                        Some(prev_part) if prev_part == cpe.part => {}
                        Some(prev_part) => {
                            counter!("repology_vulnupdater_processor_cve_configurations_total", "status" => "skipped", "skip_reason" => "mixed type CPE").increment(1);
                            info!(one = ?prev_part, two = ?cpe.part, "rejecting non-trivial Configuration with node of mixed-type CPEs");
                            return;
                        }
                        None => {
                            node_part = Some(cpe.part);
                        }
                    };
                }

                match node_part {
                    Some(crate::cpe::Part::Applications) => {
                        application_nodes.push(node);
                    }
                    Some(crate::cpe::Part::OperatingSystems) => {
                        operating_system_nodes.push(node);
                    }
                    other => {
                        counter!("repology_vulnupdater_processor_cve_configurations_total", "status" => "skipped", "skip_reason" => "bad CPE part").increment(1);
                        info!(type = ?other, "rejecting non-trivial Configuration with node of unexpected type");
                        return;
                    }
                }
            }

            if application_nodes.len() == 1 && operating_system_nodes.len() == 1 {
                counter!("repology_vulnupdater_processor_cve_configurations_total", "status" => "processed non-trivial").increment(1);
                info!("accepting non-trivial Configuration: app limited with os");
                for node in application_nodes {
                    self.collect_from_node(node);
                }
            } else {
                counter!("repology_vulnupdater_processor_cve_configurations_total", "status" => "unsupported tot-trivial").increment(1);
                info!("rejecting unsupported non-trivial Configuration");
            }
        } else {
            counter!("repology_vulnupdater_processor_cve_configurations_total", "status" => "processed trivial").increment(1);
            for node in &configuration.nodes {
                self.collect_from_node(node)
            }
        }
    }

    #[tracing::instrument(skip_all, fields(cve_id = cve.id))]
    pub fn from_cve(cve: &'a schema::Cve<'a>) -> Self {
        let mut res: Self = Default::default();
        for configuration in &cve.configurations {
            res.collect_from_configuration(configuration);
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

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    use schema::*;

    #[test]
    fn test_basic() {
        let cve = Cve {
            id: "CVE-2022-23935",
            published: "1900-01-01T00:00:00.000",
            last_modified: "1900-01-01T00:00:00.000",
            configurations: vec![Configuration {
                operator: None,
                nodes: vec![Node {
                    operator: "OR",
                    negate: false,
                    cpe_match: vec![CpeMatch {
                        vulnerable: true,
                        criteria: "cpe:2.3:a:exiftool_project:exiftool:*:*:*:*:*:*:*:*".into(),
                        version_start_including: None,
                        version_start_excluding: None,
                        version_end_including: None,
                        version_end_excluding: None,
                    }],
                }],
            }],
        };

        let matches = CpeMatches::from_cve(&cve);

        assert_eq!(matches.matches.len(), 1);
    }

    #[test]
    fn test_with_os_limit() {
        let cve = Cve {
            id: "CVE-2026-3102",
            published: "1900-01-01T00:00:00.000",
            last_modified: "1900-01-01T00:00:00.000",
            configurations: vec![Configuration {
                operator: Some("AND"),
                nodes: vec![
                    Node {
                        operator: "OR",
                        negate: false,
                        cpe_match: vec![CpeMatch {
                            vulnerable: true,
                            criteria: "cpe:2.3:a:exiftool_project:exiftool:*:*:*:*:*:*:*:*".into(),
                            version_start_including: None,
                            version_start_excluding: None,
                            version_end_including: None,
                            version_end_excluding: None,
                        }],
                    },
                    Node {
                        operator: "OR",
                        negate: false,
                        cpe_match: vec![CpeMatch {
                            vulnerable: true,
                            criteria: "cpe:2.3:o:apple:macos:-:*:*:*:*:*:*:*".into(),
                            version_start_including: None,
                            version_start_excluding: None,
                            version_end_including: None,
                            version_end_excluding: None,
                        }],
                    },
                ],
            }],
        };

        let matches = CpeMatches::from_cve(&cve);

        assert_eq!(matches.matches.len(), 1);
    }

    #[test]
    fn test_and_not_supported() {
        let cve = Cve {
            id: "CVE-2026-3102",
            published: "1900-01-01T00:00:00.000",
            last_modified: "1900-01-01T00:00:00.000",
            configurations: vec![Configuration {
                operator: Some("AND"),
                nodes: vec![
                    Node {
                        operator: "OR",
                        negate: false,
                        cpe_match: vec![CpeMatch {
                            vulnerable: true,
                            criteria: "cpe:2.3:a:exiftool_project:exiftool:*:*:*:*:*:*:*:*".into(),
                            version_start_including: None,
                            version_start_excluding: None,
                            version_end_including: None,
                            version_end_excluding: None,
                        }],
                    },
                    Node {
                        operator: "OR",
                        negate: false,
                        cpe_match: vec![CpeMatch {
                            vulnerable: true,
                            criteria: "cpe:2.3:a:exiftool_project:exiftool:*:*:*:*:*:*:*:*".into(),
                            version_start_including: None,
                            version_start_excluding: None,
                            version_end_including: None,
                            version_end_excluding: None,
                        }],
                    },
                ],
            }],
        };

        let matches = CpeMatches::from_cve(&cve);

        assert_eq!(matches.matches.len(), 0);
    }
}
