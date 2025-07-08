// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use anyhow::{Context, anyhow, bail};

use repology_common::LinkType;

use crate::parsing::package_maker::{NameType, PackageMaker};
use crate::parsing::parser::{PackageParser, PackageSink};
use crate::parsing::utils::maintainers::extract_maintainers;
use crate::parsing::utils::walk::{WalkEntry, WalkFileTree};

#[allow(unused)]
mod data {
    use std::collections::HashMap;

    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Meta {
        pub version: String,
        pub maintainer: String,
        pub sources: Vec<String>,
        pub checksums: Vec<String>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Package {
        pub meta: Meta,
        pub deps: HashMap<String, String>,
        pub mkdeps: HashMap<String, String>,
    }
}

pub struct TinCanParser {}

impl TinCanParser {
    fn process_package(entry: &WalkEntry, sink: &mut dyn PackageSink) -> anyhow::Result<()> {
        let package_path_absolute = entry
            .path_absolute
            .parent()
            .expect("should be able to get parent path of package.toml");
        let package_subdir = package_path_absolute
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .ok_or_else(|| anyhow!("cannot extract name of package subdirectory"))?;

        let pkgdata: data::Package =
            toml::from_str(std::str::from_utf8(&std::fs::read(&entry.path_absolute)?)?)?;

        let mut pkg = PackageMaker::default();

        pkg.set_names(
            package_subdir,
            NameType::SrcName
                | NameType::TrackName
                | NameType::DisplayName
                | NameType::ProjectNameSeed,
        );
        pkg.set_version(pkgdata.meta.version);
        pkg.add_maintainers(extract_maintainers(&pkgdata.meta.maintainer));

        let mut patches = vec![];
        for source in &pkgdata.meta.sources {
            if source.contains("://") {
                pkg.add_link(LinkType::UpstreamDownload, source);
            } else if source.starts_with("files/") && source.ends_with(".patch") {
                if !std::fs::exists(package_path_absolute.join(source))? {
                    bail!(
                        "patch file {source} mentioned in meta.sources section was not found on the file system"
                    );
                } else {
                    patches.push(source);
                }
            }
        }

        pkg.set_extra_field_many("patches", patches);

        Ok(sink.push(pkg)?)
    }
}

impl PackageParser for TinCanParser {
    fn parse(&self, path: &Path, sink: &mut dyn PackageSink) -> anyhow::Result<()> {
        for entry in WalkFileTree::walk_by_name(path, "package.toml") {
            let entry = entry?;
            Self::process_package(&entry, sink)
                .with_context(|| format!("while processing {:?}", entry.path_relative))?;
        }

        Ok(())
    }
}
