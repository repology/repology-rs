// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Debug;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

fn from_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or a sequence of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(value) = seq.next_element::<String>()? {
                vec.push(value);
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_any(Visitor)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryLink {
    pub desc: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageLink {
    pub r#type: String,
    pub url: String,
    pub priority: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    #[serde(deserialize_with = "from_string_or_vec")]
    pub name: Vec<String>,
    pub fetcher: serde_json::Value,
    pub parser: serde_json::Value,
    pub subrepo: Option<String>,
    #[serde(default)]
    pub packagelinks: Vec<PackageLink>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repository {
    pub name: String,
    pub sortname: Option<String>,
    pub r#type: String,
    pub desc: String,
    pub statsgroup: Option<String>,
    pub singular: Option<String>,
    pub family: String,
    #[serde(deserialize_with = "from_string_or_vec")]
    pub ruleset: Vec<String>,
    pub color: Option<String>,
    pub minpackages: u64,
    pub update_period: Option<String>,
    pub pessimized: Option<String>,
    pub valid_till: Option<String>,
    pub default_maintainer: Option<String>,
    pub sources: Vec<Source>,
    #[serde(default)]
    pub shadow: bool,
    #[serde(default)]
    pub incomplete: bool,
    #[serde(default)]
    pub repolinks: Vec<RepositoryLink>,
    #[serde(default)]
    pub packagelinks: Vec<PackageLink>,
    pub groups: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoriesConfig {
    pub repositories: Vec<Repository>,
}

impl RepositoriesConfig {
    pub fn parse(path: &Path) -> anyhow::Result<Self> {
        let mut repositories: Vec<Repository> = vec![];

        for entry in WalkDir::new(path).sort_by_file_name().into_iter() {
            let entry = entry?;
            if entry.file_type().is_file()
                && entry
                    .file_name()
                    .to_str()
                    .is_some_and(|filename| filename.ends_with(".yaml"))
            {
                let template = std::fs::read_to_string(entry.path()).with_context(|| {
                    format!(
                        "failed to read repositories config {}",
                        entry.path().display()
                    )
                })?;

                let use_tera = template.contains("# parse_with_tera")
                    && !template.contains("# parse_with_jinja");

                let yaml: String = if use_tera {
                    tera::Tera::one_off(&template, &tera::Context::new(), false).with_context(
                        || {
                            format!(
                                "failed to preprocess repositories config with tera {}",
                                entry.path().display()
                            )
                        },
                    )?
                } else {
                    use pyo3::ffi::c_str;
                    use pyo3::prelude::*;
                    use pyo3::types::{PyDict, PyString};

                    Python::attach(|py| -> PyResult<String> {
                        let locals = PyDict::new(py);
                        locals.set_item("jinja2", py.import("jinja2")?)?;
                        locals.set_item("input", PyString::from_bytes(py, template.as_bytes())?)?;
                        let code = c_str!("jinja2.Template(input).render()");
                        let output: String = py.eval(code, None, Some(&locals))?.extract()?;

                        Ok(output)
                    })
                    .with_context(|| {
                        format!(
                            "failed to preprocess repositories config with python {}",
                            entry.path().display()
                        )
                    })?
                };

                let mut chunk: Vec<Repository> =
                    serde_saphyr::from_str(&yaml).with_context(|| {
                        format!(
                            "failed to parse repositories config {}",
                            entry.path().display()
                        )
                    })?;
                repositories.append(&mut chunk);
            }
        }

        Ok(Self { repositories })
    }

    pub fn to_yaml(&self) -> anyhow::Result<String> {
        Ok(serde_saphyr::to_string(&self.repositories)?)
    }
}
