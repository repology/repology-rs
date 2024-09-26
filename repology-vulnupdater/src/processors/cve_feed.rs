// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod mitre;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::str::{from_utf8, FromStr};

use anyhow::Error;
use async_trait::async_trait;
use sqlx::PgPool;
use tokio::io::AsyncReadExt;

use crate::cpe::Cpe;
use crate::processors::{DatasourceProcessor, DatasourceUpdateResult};

#[derive(PartialEq, Eq, Hash, Debug)]
struct VersionBound<'a> {
    version: Cow<'a, str>,
    excluded: bool,
}

impl<'a> VersionBound<'a> {
    pub fn new(including: Option<Cow<'a, str>>, excluding: Option<Cow<'a, str>>) -> Option<Self> {
        // TODO: report if included.is_some() and excluded.is_some()?
        if let Some(version) = including {
            Some(Self {
                version,
                excluded: false,
            })
        } else if let Some(version) = excluding {
            Some(Self {
                version,
                excluded: true,
            })
        } else {
            None
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct VersionRange<'a> {
    start: Option<VersionBound<'a>>,
    end: Option<VersionBound<'a>>,
}

impl<'a> VersionRange<'a> {
    pub fn from_single_version(version: String) -> Self {
        Self {
            start: Some(VersionBound {
                version: Cow::from(version.clone()),
                excluded: false,
            }),
            end: Some(VersionBound {
                version: Cow::from(version),
                excluded: false,
            }),
        }
    }
}

fn parse_datetime(date: &str) -> Result<chrono::NaiveDateTime, Error> {
    Ok(chrono::NaiveDateTime::parse_from_str(
        date,
        "%Y-%m-%dT%H:%MZ",
    )?)
}

pub struct CveFeedProcessor {
    pool: PgPool,
}

impl CveFeedProcessor {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DatasourceProcessor for CveFeedProcessor {
    async fn process(
        &self,
        mut data: Box<dyn tokio::io::AsyncRead + Send + Unpin>,
    ) -> Result<DatasourceUpdateResult, Error> {
        let mut buffer: Vec<u8> = Default::default();
        data.read_to_end(&mut buffer).await?;
        let json_text = from_utf8(buffer.as_ref())?;
        let feed = serde_json::from_str::<self::mitre::Feed>(json_text)?;
        let mut total_num_changes: u64 = 0;

        let mut tx = self.pool.begin().await?;

        for item in feed.cve_items {
            let mut matches: HashMap<Cpe, HashSet<VersionRange>> = Default::default();

            for node in item.configurations.nodes {
                if node.operator != "OR" {
                    // TODO: complex configurations not supported yet
                    continue;
                }

                for cpe_match in node.cpe_match {
                    if !cpe_match.vulnerable {
                        // TODO: investigate if vulnerability exclusions are relevant
                        continue;
                    }

                    let cpe = if let Ok(cpe) = Cpe::from_str(&cpe_match.cpe) {
                        cpe
                    } else {
                        // TODO: log or fix these cases
                        continue;
                    };

                    let good_part = cpe.part == crate::cpe::Part::Applications
                        || cpe.part == crate::cpe::Part::OperatingSystems
                            && (cpe.vendor == "linux"
                                && (cpe.product == "linux" || cpe.product == "linux_kernel")
                                || cpe.vendor == "xen" && cpe.product == "xen");
                    if !good_part {
                        // TODO: recheck if we need this condition
                        continue;
                    }

                    if cpe.version == "-" {
                        // TODO: log ?
                        continue;
                    }

                    if cpe.version == "*" {
                        // version defined by ranges
                        let start = VersionBound::new(
                            cpe_match.version_start_including,
                            cpe_match.version_start_excluding,
                        );
                        let end = VersionBound::new(
                            cpe_match.version_end_including,
                            cpe_match.version_end_excluding,
                        );

                        matches
                            .entry(cpe.clone())
                            .or_default()
                            .insert(VersionRange { start, end });
                    } else {
                        let version = if cpe.update != "*" && cpe.update != "-" && !cpe.update.is_empty()
                        {
                            if cpe.update.starts_with(|c: char| c.is_ascii_digit()) {
                                format!("{}-{}", cpe.version, cpe.update)
                            } else {
                                format!("{}{}", cpe.version, cpe.update)
                            }
                        } else {
                            cpe.version.clone()
                        };

                        matches
                            .entry(cpe.clone())
                            .or_default()
                            .insert(VersionRange::from_single_version(version));

                        // XXX: log if any of version_{start,end}_{including,excluding} is also
                        // defined here
                    }
                }
            }

            let vendor_product_pairs: Vec<String> = matches
                .keys()
                .map(|cpe| format!("{}:{}", cpe.vendor, cpe.product))
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            let matches_as_json: serde_json::Value = matches
                .into_iter()
                .map(|(cpe, ranges)| ranges.into_iter().map(move |range| (cpe.clone(), range)))
                .flatten()
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
                .collect();

            total_num_changes += sqlx::query(
                r#"
            WITH updated_cves AS (
                INSERT INTO cves (
                    cve_id,
                    published,
                    last_modified,
                    matches,
                    cpe_pairs
                )
                VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5
                )
                ON CONFLICT(cve_id) DO UPDATE
                SET
                    published = $2,  -- not expected to change in fact
                    last_modified = $3,
                    matches = $4,
                    cpe_pairs = $5
                WHERE
                    $3 > cves.last_modified
                RETURNING cpe_pairs
            )
            INSERT INTO cpe_updates (
                cpe_vendor,
                cpe_product
            )
            SELECT
                split_part(unnest(cpe_pairs), ':', 1) AS cpe_vendor,
                split_part(unnest(cpe_pairs), ':', 2) AS cpe_product
            FROM
                updated_cves
            "#,
            )
            .bind(&item.cve.cve_meta_data.id)
            .bind(parse_datetime(item.published_date)?)
            .bind(parse_datetime(item.last_modified_date)?)
            .bind(
                if matches_as_json.as_array().is_some_and(|v| !v.is_empty()) {
                    Some(matches_as_json)
                } else {
                    None
                },
            )
            .bind(if vendor_product_pairs.is_empty() {
                None
            } else {
                Some(vendor_product_pairs)
            })
            .execute(&mut *tx)
            .await?
            .rows_affected();
        }

        tx.commit().await?;

        Ok(DatasourceUpdateResult::HadChanges(total_num_changes))
    }
}
