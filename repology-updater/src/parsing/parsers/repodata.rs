// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;
use std::path::Path;

use anyhow::bail;
use serde::Deserialize;

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
        //#[serde(rename = "rpm:provides")]
        //provides
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
}

impl Default for RepodataParserOptions {
    fn default() -> Self {
        Self {
            allow_src: true,
            allow_bin: true,
            disttags: vec![],
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
    ) -> anyhow::Result<()> {
        let mut pkg = PackageMaker::default();

        if pkgdata.name.contains("%{") {
            bail!(
                "unexpanded interpolation in package name ({})",
                pkgdata.name
            );
        }

        let is_src = pkgdata.arch == "src";

        if is_src {
            if !self.options.allow_src {
                return Ok(());
            }

            pkg.set_names(
                pkgdata.name,
                NameType::SrcName
                    | NameType::TrackName
                    | NameType::DisplayName
                    | NameType::ProjectNameSeed,
            );
        } else {
            if !self.options.allow_bin {
                return Ok(());
            }

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

        // TODO: provides -> binnames

        Ok(process(pkg)?)
    }
}

impl PackageParser for RepodataParser {
    #[tracing::instrument(name = "PackageParser", skip_all, fields(options = ?self.options))]
    fn parse(&self, path: &Path, process: &mut dyn PackageProcessor) -> anyhow::Result<()> {
        let metadata: data::Metadata = serde_xml_rs::from_reader(File::open_buffered(path)?)?;

        // TODO: statistics

        for package in metadata.packages {
            self.process_package(package, process)?;
        }

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
