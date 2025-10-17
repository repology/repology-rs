// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::BufRead as _;
use std::path::Path;

use anyhow::{Context, bail};

use repology_common::LinkType;

use crate::parsing::package_maker::{NameType, PackageMaker};
use crate::parsing::parser::{PackageParser, PackageSink};
use crate::parsing::utils::maintainers::extract_maintainers;

#[allow(unused)]
mod data {
    use std::collections::HashMap;

    use serde::Deserialize;

    #[derive(Deserialize)]
    #[cfg_attr(feature = "strict-parsers", serde(deny_unknown_fields))]
    pub struct Package {
        pub category: String,
        pub ix_pkg_fs_name: String,
        pub ix_pkg_name: String,
        pub ix_pkg_full_name: String,
        pub pkg_name: String,
        pub pkg_ver: String,
        pub recipe: String,
        pub maintainers: Vec<String>,
        pub upstream_urls: Vec<String>,
        pub lang_module: Option<String>,
    }
}

pub struct StalIxParser {}

impl StalIxParser {
    fn parse_line(line: &str, sink: &mut dyn PackageSink) -> anyhow::Result<()> {
        let pkgdata: data::Package = serde_json::from_str(line)?;

        // a hack because we have no way to pass arbitrary flag to the ruleset
        // in fact the prefix should be added by 500ths rules
        let prefix = match pkgdata.lang_module.as_deref() {
            Some("python") => "python:",
            Some("perl") => "perl:",
            None => "",
            Some(other) => {
                bail!("unexpected lang_module {other}");
            }
        };

        let mut pkg = PackageMaker::default();

        pkg.set_names(
            pkgdata.ix_pkg_full_name,
            NameType::SrcName | NameType::TrackName,
        );
        pkg.set_names(
            pkgdata.ix_pkg_name,
            NameType::BinName | NameType::DisplayName,
        );
        pkg.set_names(
            format!("{prefix}{}", pkgdata.pkg_name),
            NameType::ProjectNameSeed,
        );

        pkg.set_version(pkgdata.pkg_ver);
        pkg.add_category(pkgdata.category);
        for maintainer in &pkgdata.maintainers {
            pkg.add_maintainers(extract_maintainers(maintainer));
        }
        for url in &pkgdata.upstream_urls {
            pkg.add_link(LinkType::UpstreamDownload, url);
        }

        Ok(sink.push(pkg)?)
    }
}

impl PackageParser for StalIxParser {
    fn parse(&self, path: &Path, sink: &mut dyn PackageSink) -> anyhow::Result<()> {
        let f = std::fs::File::open_buffered(path)?;

        for (nline, line) in f.lines().enumerate() {
            Self::parse_line(&line?, sink).with_context(|| format!("on line {}", nline + 1))?;
        }

        Ok(())
    }
}
