// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Root<'a> {
    #[serde(borrow)]
    pub vulnerabilities: Vec<Vulnerability<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct Vulnerability<'a> {
    #[serde(borrow)]
    pub cve: Cve<'a>,
}

#[derive(Deserialize, Debug)]
pub struct Cve<'a> {
    #[serde(rename = "id", borrow)]
    pub id: &'a str,
    #[serde(rename = "published", borrow)]
    pub published: &'a str,
    #[serde(rename = "lastModified", borrow)]
    pub last_modified: &'a str,
    #[serde(borrow, default)]
    pub configurations: Vec<Configuration<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct Configuration<'a> {
    #[serde(borrow)]
    pub nodes: Vec<Node<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct Node<'a> {
    #[serde(borrow)]
    pub operator: &'a str,
    pub negate: bool,
    #[serde(rename = "cpeMatch", borrow)]
    pub cpe_match: Vec<CpeMatch<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct CpeMatch<'a> {
    pub vulnerable: bool,
    pub criteria: Cow<'a, str>,
    #[serde(rename = "versionStartIncluding", borrow)]
    pub version_start_including: Option<Cow<'a, str>>,
    #[serde(rename = "versionStartExcluding", borrow)]
    pub version_start_excluding: Option<Cow<'a, str>>,
    #[serde(rename = "versionEndIncluding", borrow)]
    pub version_end_including: Option<Cow<'a, str>>,
    #[serde(rename = "versionEndExcluding", borrow)]
    pub version_end_excluding: Option<Cow<'a, str>>,
}
