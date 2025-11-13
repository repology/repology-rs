// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::bail;
use serde::Deserialize;
use tracing::{error, info, warn};

use repology_common::LinkType;

use crate::parsing::package_maker::{NameType, PackageMaker};
use crate::parsing::parser::{PackageParser, PackageProcessor};
use crate::parsing::utils::maintainers::extract_maintainers;
use crate::parsing::utils::nevra::{Nevra, merge_evr};
use crate::parsing::utils::rpm::normalize_rpm_version;

#[allow(unused)]
mod data {
    use std::collections::HashMap;

    use serde::Deserialize;

    #[derive(Deserialize)]
    #[cfg_attr(feature = "strict-parsers", serde(deny_unknown_fields))]
    pub struct Entry {
        #[serde(rename = "@name")]
        pub name: String,
        #[serde(rename = "@flags")]
        pub flags: Option<String>,
        #[serde(rename = "@epoch")]
        pub epoch: Option<String>,
        #[serde(rename = "@ver")]
        pub ver: Option<String>,
        #[serde(rename = "@rel")]
        pub rel: Option<String>,
    }

    #[derive(Deserialize, Default)]
    #[cfg_attr(feature = "strict-parsers", serde(deny_unknown_fields))]
    pub struct Provides {
        #[serde(rename = "rpm:entry")]
        pub entries: Vec<Entry>,
    }

    #[derive(Deserialize)]
    //#[cfg_attr(feature = "strict-parsers", serde(deny_unknown_fields))]
    pub struct Format {
        #[serde(rename = "rpm:license")]
        pub license: String,
        //#[serde(rename = "rpm:vendor")]
        //pub vendor: String,
        #[serde(rename = "rpm:group")]
        pub group: String,
        #[serde(rename = "rpm:sourcerpm")]
        pub sourcerpm: String,
        //#[serde(rename = "rpm:header-range")]
        //header-range
        #[serde(rename = "rpm:provides", default)]
        pub provides: Provides,
        //#[serde(rename = "rpm:required")]
        //required
    }

    #[derive(Deserialize)]
    #[cfg_attr(feature = "strict-parsers", serde(deny_unknown_fields))]
    pub struct Version {
        #[serde(rename = "@epoch")]
        pub epoch: String,
        #[serde(rename = "@ver")]
        pub ver: String,
        #[serde(rename = "@rel")]
        pub rel: String,
    }

    #[derive(Deserialize)]
    //#[cfg_attr(feature = "strict-parsers", serde(deny_unknown_fields))]
    pub struct Package {
        #[serde(rename = "@type")]
        pub r#type: String,
        pub name: String,
        pub arch: String,
        pub version: Version,
        // pub checksum
        pub summary: String,
        // pub description
        pub packager: String,
        pub url: String,
        // pub time
        // pub size
        // pub location
        pub format: Format,
    }

    #[derive(Deserialize)]
    #[cfg_attr(feature = "strict-parsers", serde(deny_unknown_fields))]
    pub struct Metadata {
        #[serde(rename = "package")]
        pub packages: Vec<Package>,
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct RepodataParserOptions {
    pub allow_src: bool,
    pub allow_bin: bool,
    pub disttags: Vec<String>,
    pub binnames_from_provides: bool,
}

impl Default for RepodataParserOptions {
    fn default() -> Self {
        Self {
            allow_src: true,
            allow_bin: true,
            disttags: vec![],
            binnames_from_provides: true,
        }
    }
}

#[derive(Default)]
struct Statistics {
    pub skipped_archs: HashMap<String, u64>,
    pub has_provides: bool,
    pub num_skipped_provides_without_version: u64,
    pub skipped_provides_without_version_sample: Vec<String>,
    pub num_skipped_provides_with_parentheses: u64,
    pub skipped_provides_with_parentheses_sample: Vec<String>,
}

impl Statistics {
    pub fn push_skipped_provides_without_version(&mut self, name: String) {
        self.num_skipped_provides_without_version += 1;
        const SAMPLE_SIZE: usize = 10;
        if self.skipped_provides_without_version_sample.len() < SAMPLE_SIZE {
            self.skipped_provides_without_version_sample.push(name);
        }
    }

    pub fn push_skipped_provides_with_parentheses(&mut self, name: String) {
        self.num_skipped_provides_with_parentheses += 1;
        const SAMPLE_SIZE: usize = 10;
        if self.skipped_provides_with_parentheses_sample.len() < SAMPLE_SIZE {
            self.skipped_provides_with_parentheses_sample.push(name);
        }
    }

    pub fn trace(&self, options: &RepodataParserOptions) {
        for (arch, count) in &self.skipped_archs {
            info!(count, arch, "skipped packages with disallowed architecture");
        }

        if self.has_provides && !options.binnames_from_provides {
            error!(
                "not extracting binary package names from <rpm:provides> entries for this repository, because explicitly disabled in config"
            );
        }

        if self.num_skipped_provides_without_version > 0 {
            warn!(
                count = self.num_skipped_provides_without_version,
                sample = self
                    .skipped_provides_without_version_sample
                    .iter()
                    .map(String::as_ref)
                    .intersperse(", ")
                    .collect::<String>(),
                "skipped <rpm:provides> entries with incomplete version (rel/ver)"
            )
        }

        if self.num_skipped_provides_with_parentheses > 0 {
            warn!(
                count = self.num_skipped_provides_with_parentheses,
                sample = self
                    .skipped_provides_with_parentheses_sample
                    .iter()
                    .map(String::as_ref)
                    .intersperse(", ")
                    .collect::<String>(),
                "skipped <rpm:provides> entries with parentheses"
            )
        }
    }
}

pub struct RepodataParser {
    options: RepodataParserOptions,
}

impl RepodataParser {
    pub fn new(options: RepodataParserOptions) -> Self {
        Self { options }
    }

    fn process_package(
        &self,
        pkgdata: data::Package,
        process: &mut dyn PackageProcessor,
        statistics: &mut Statistics,
    ) -> anyhow::Result<()> {
        let mut pkg = PackageMaker::default();

        if pkgdata.name.contains("%{") {
            bail!(
                "unexpanded interpolation in package name ({})",
                pkgdata.name
            );
        }

        let is_src = pkgdata.arch == "src";

        let skip_arch = if is_src {
            !self.options.allow_src
        } else {
            !self.options.allow_bin
        };

        if skip_arch {
            *statistics.skipped_archs.entry(pkgdata.arch).or_default() += 1;
            return Ok(());
        }

        if is_src {
            pkg.set_names(
                pkgdata.name,
                NameType::SrcName
                    | NameType::TrackName
                    | NameType::DisplayName
                    | NameType::ProjectNameSeed,
            );
        } else {
            pkg.set_names(pkgdata.name, NameType::BinName | NameType::DisplayName);
            pkg.set_names(
                Nevra::parse(&pkgdata.format.sourcerpm)?.name,
                NameType::SrcName | NameType::TrackName | NameType::ProjectNameSeed,
            );
        }

        let (normalized_version, flags) = normalize_rpm_version(
            &pkgdata.version.ver,
            &pkgdata.version.rel,
            &self.options.disttags,
        );
        let raw_version = merge_evr(
            Some(&pkgdata.version.epoch),
            &pkgdata.version.ver,
            &pkgdata.version.rel,
        );

        pkg.set_version_with_raw(normalized_version, raw_version);
        pkg.set_flags(flags);

        pkg.set_summary(pkgdata.summary);
        pkg.add_link(LinkType::UpstreamHomepage, pkgdata.url);
        pkg.add_category(pkgdata.format.group);
        pkg.add_license(pkgdata.format.license);
        pkg.set_arch(pkgdata.arch);
        pkg.add_maintainers(extract_maintainers(&pkgdata.packager));

        if is_src {
            // Provides can contain all kinds of garbage apart from binary packages,
            // especially in Terra repositories. Examples:
            //   <rpm:entry name="rpm_macro(_sccache)"/>
            //   <rpm:entry name="libapparmor.so.1.18.0-4.0.2-1.fc41.aarch64.debug()(64bit)"/>
            //   <rpm:entry name="pkgconfig(libapparmor)" flags="EQ" epoch="0" ver="4.0.2"/>
            //   <rpm:entry name="debuginfo(build-id)" flags="EQ" epoch="0" ver="06b5e8418ffeef59bc0d31a91900ebdd8bddc2b5"/>
            // We only try to extract entries which are likely to be binary package names,
            // e.g. entries with version defined and without parentheses in the name.
            for provides in pkgdata.format.provides.entries {
                statistics.has_provides = true;

                if !self.options.binnames_from_provides {
                    continue;
                }

                // epoch is optional for e.g. openmandriva
                if provides.rel.is_none() || provides.ver.is_none() {
                    statistics.push_skipped_provides_without_version(provides.name);
                    continue;
                }
                if provides.name.contains('(') {
                    statistics.push_skipped_provides_with_parentheses(provides.name);
                    continue;
                }
                pkg.add_binname(provides.name);
            }
        }

        Ok(process(pkg)?)
    }
}

impl PackageParser for RepodataParser {
    #[tracing::instrument(name = "PackageParser", skip_all, fields(options = ?self.options))]
    fn parse(&self, path: &Path, process: &mut dyn PackageProcessor) -> anyhow::Result<()> {
        let metadata: data::Metadata = serde_xml_rs::from_reader(File::open_buffered(path)?)?;

        let mut statistics = Statistics::default();

        for package in metadata.packages {
            self.process_package(package, process, &mut statistics)?;
        }

        statistics.trace(&self.options);

        Ok(())
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    parser_test!(
        RepodataParser::new(RepodataParserOptions {
            disttags: vec!["el".to_string()],
            ..Default::default()
        }),
        repodata,
        ok_centos
    );
    parser_test!(
        RepodataParser::new(RepodataParserOptions {
            disttags: vec!["fc".to_string()],
            ..Default::default()
        }),
        repodata,
        ok_fedora
    );
    parser_test!(
        RepodataParser::new(RepodataParserOptions {
            disttags: vec!["fc".to_string()],
            ..Default::default()
        }),
        repodata,
        ok_rpmfusion
    );
    parser_test!(
        RepodataParser::new(RepodataParserOptions {
            disttags: vec!["fcrawhide".to_string()],
            binnames_from_provides: false,
            ..Default::default()
        }),
        repodata,
        ok_terra
    );
    parser_test!(
        RepodataParser::new(RepodataParserOptions {
            disttags: vec!["mamba".to_string()],
            ..Default::default()
        }),
        repodata,
        ok_openmamba
    );
}
