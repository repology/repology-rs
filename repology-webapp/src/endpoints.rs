// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use anyhow::{anyhow, bail, Error};
use serde_json::Value;
use strum::EnumProperty;
use strum_macros::{EnumString, IntoStaticStr};

#[derive(EnumProperty, IntoStaticStr, EnumString)]
pub enum Endpoint {
    #[strum(props(path = "/api/v1/project/:project_name"))]
    ApiV1Project,
    #[strum(props(path = "/badge/tiny-repos/:project_name.svg"))]
    BadgeTinyRepos,
    #[strum(props(path = "/badge/version-for-repo/:repository_name/:project_name.svg"))]
    BadgeVersionForRepo,
    #[strum(props(path = "/badge/vertical-allrepos/:project_name.svg"))]
    BadgeVerticalAllRepos,
    #[strum(props(path = "/badge/latest-versions/:project_name.svg"))]
    BadgeLatestVersions,
    #[strum(props(path = "/static/:file_name"))]
    StaticFile,
}

#[derive(EnumString, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Admin,
    Docs,
    Experimental,
    Maintainers,
    News,
    Projects,
    Repositories,
    Security,
    Tools,
}

impl Endpoint {
    pub fn path(&self) -> &'static str {
        self.get_str("path")
            .expect("path should exist for the endpoint")
    }

    pub fn name(&self) -> &'static str {
        self.into()
    }

    pub fn is_section(&self, section: Section) -> bool {
        use std::str::FromStr as _;
        self.get_str("section")
            .is_some_and(|endpoint_section| Section::from_str(endpoint_section).unwrap() == section)
    }

    pub fn construct_url(&self, values: &HashMap<String, Value>) -> Result<String, Error> {
        let is_key_char = |c: char| c.is_lowercase() || c == '_';

        let mut rest = self.path();
        let mut res = String::new();

        loop {
            if let Some((prefix, key_and_rest)) = rest.split_once(":") {
                res += prefix;

                let (key, suffix) = key_and_rest.split_at(
                    key_and_rest
                        .find(|c| !is_key_char(c))
                        .unwrap_or(key_and_rest.len()),
                );

                match values
                    .get(key)
                    .ok_or(anyhow!("missing value for path placeholder \"{}\"", key))?
                {
                    Value::Number(n) => {
                        res += &n.to_string();
                    }
                    Value::String(s) => {
                        res += s;
                    }
                    _ => {
                        bail!("invalid value type for path placeholder \"{}\"", key);
                    }
                };
                rest = suffix;
            } else {
                return Ok(res + rest);
            }
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(
            Endpoint::BadgeVersionForRepo.path(),
            "/badge/version-for-repo/:repository_name/:project_name.svg"
        );
    }

    #[test]
    fn test_name() {
        assert_eq!(Endpoint::BadgeVersionForRepo.name(), "BadgeVersionForRepo");
    }

    #[test]
    fn test_construct_url() {
        use serde_json::json;
        let repository_name: HashMap<String, Value> = [("repository_name".into(), json!("foo"))]
            .into_iter()
            .collect();
        let project_name: HashMap<String, Value> = [("project_name".into(), json!("bar"))]
            .into_iter()
            .collect();
        let project_and_repository_names = {
            let mut t = project_name.clone();
            t.extend(repository_name.clone().into_iter());
            t
        };

        assert_eq!(
            Endpoint::ApiV1Project.construct_url(&project_name).unwrap(),
            "/api/v1/project/bar"
        );

        assert_eq!(
            Endpoint::BadgeVersionForRepo
                .construct_url(&project_and_repository_names)
                .unwrap(),
            "/badge/version-for-repo/foo/bar.svg"
        );
        assert!(Endpoint::BadgeVersionForRepo
            .construct_url(&repository_name)
            .is_err());
    }
}
