// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use anyhow::Context;

use repology_common::LinkType;

use crate::parsing::package_maker::{NameType, PackageMaker};
use crate::parsing::parser::PackageParser;
use crate::parsing::sink::PackageSink;

#[allow(unused)]
mod data {
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[cfg_attr(feature = "strict-parsers", serde(deny_unknown_fields))]
    pub struct SubPackage {
        pub name: String,
        pub category: Vec<String>,
    }

    #[derive(Deserialize)]
    #[cfg_attr(feature = "strict-parsers", serde(deny_unknown_fields))]
    pub struct Package {
        pub name: String,
        pub version: String,
        pub category: Vec<String>,
        pub summary: String,
        pub homepage: String,
        pub subpackages: Vec<SubPackage>,
        pub maintainers: Vec<String>,
    }

    #[derive(Deserialize)]
    #[cfg_attr(feature = "strict-parsers", serde(deny_unknown_fields))]
    pub struct Data {
        pub repository_name: String,
        pub num_packages: usize,
        pub timestamp: u64,
        pub packages: Vec<Package>,
    }
}

pub struct YacpParser {}

impl YacpParser {
    fn process_package(package: data::Package, sink: &mut dyn PackageSink) -> anyhow::Result<()> {
        let mut pkg = PackageMaker::default();

        pkg.set_names(
            package.name,
            NameType::SrcName
                | NameType::TrackName
                | NameType::DisplayName
                | NameType::ProjectNameSeed,
        );
        pkg.set_version(package.version);
        pkg.add_categories(package.category);
        pkg.set_summary(package.summary);
        pkg.add_link(LinkType::UpstreamHomepage, package.homepage);
        pkg.add_binnames(
            package
                .subpackages
                .into_iter()
                .map(|subpackage| subpackage.name),
        );

        Ok(sink.push(pkg)?)
    }
}

impl PackageParser for YacpParser {
    fn parse(&self, path: &Path, sink: &mut dyn PackageSink) -> anyhow::Result<()> {
        let data: data::Data = serde_json::from_reader(std::fs::File::open_buffered(path)?)?;

        for (npackage, package) in data.packages.into_iter().enumerate() {
            Self::process_package(package, sink)
                .with_context(|| format!("entry #{}", npackage + 1))?;
        }

        Ok(())
    }
}
