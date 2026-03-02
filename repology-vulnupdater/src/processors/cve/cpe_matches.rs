// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use metrics::counter;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::str::FromStr as _;

use tracing::warn;

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
            warn!("unsupported negated node");
            return;
        }

        counter!("repology_vulnupdater_processor_cve_nodes_total", "status" => "processed")
            .increment(1);

        for cpe_match in &node.cpe_match {
            counter!("repology_vulnupdater_processor_cve_cpes_total").increment(1);

            if !cpe_match.vulnerable {
                // CpeMatches serving as a limit and not denoting a vulnerability,
                continue;
            }

            let Ok(cpe) = Cpe::from_str(&cpe_match.criteria) else {
                counter!("repology_vulnupdater_processor_cve_cpes_total", "status" => "skipped", "skip_reason" => "unparsable CPE").increment(1);
                warn!(criteria = cpe_match.criteria.as_ref(), "unparsable CPE");
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

    #[tracing::instrument(skip_all, fields(cve_id = cve.id))]
    pub fn from_cve(cve: &'a schema::Cve<'a>) -> Self {
        let mut res: Self = Default::default();
        for configuration in &cve.configurations {
            // TODO: configuration has rather complex structure allowing
            // to denote, for instance, that a vulnerability only applies
            // on specific OS or hardware. We ignore it though, as we cannot
            // handle platform/arch limits for CVEs anyway - we tell CPEs
            // which refer to vulnerable products from CPEs which refer to
            // limits by vulnerable flag on CpeMatch, and that should be
            // enough for now.
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

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    use schema::*;

    #[test]
    fn test_basic() {
        let cpe = "cpe:2.3:a:foo:bar:*:*:*:*:*:*:*:*";

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
                        criteria: cpe.into(),
                        version_start_including: None,
                        version_start_excluding: None,
                        version_end_including: None,
                        version_end_excluding: None,
                    }],
                }],
            }],
        };

        assert_eq!(
            CpeMatches::from_cve(&cve)
                .matches
                .into_keys()
                .collect::<Vec<_>>(),
            vec![Cpe::from_str(cpe).unwrap()]
        );
    }

    #[test]
    fn test_with_limit() {
        let vulnerable_cpe = "cpe:2.3:a:foo:bar:*:*:*:*:*:*:*:*";
        let limiting_cpe = "cpe:2.3:o:bar:baz:*:*:*:*:*:*:*:*";

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
                            criteria: vulnerable_cpe.into(),
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
                            vulnerable: false,
                            criteria: limiting_cpe.into(),
                            version_start_including: None,
                            version_start_excluding: None,
                            version_end_including: None,
                            version_end_excluding: None,
                        }],
                    },
                ],
            }],
        };

        assert_eq!(
            CpeMatches::from_cve(&cve)
                .matches
                .into_keys()
                .collect::<Vec<_>>(),
            vec![Cpe::from_str(vulnerable_cpe).unwrap()]
        );
    }
}
