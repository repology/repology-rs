// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Feed<'a> {
    #[serde(rename = "CVE_Items", borrow)]
    pub cve_items: Vec<CveItem<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct CveItem<'a> {
    #[serde(borrow)]
    pub cve: Cve<'a>,
    #[serde(rename = "publishedDate", borrow)]
    pub published_date: &'a str,
    #[serde(rename = "lastModifiedDate", borrow)]
    pub last_modified_date: &'a str,
    #[serde(borrow)]
    pub configurations: Configurations<'a>,
}

#[derive(Deserialize, Debug)]
pub struct Cve<'a> {
    #[serde(rename = "CVE_data_meta", borrow)]
    pub cve_meta_data: CveMetaData<'a>,
}

#[derive(Deserialize, Debug)]
pub struct CveMetaData<'a> {
    #[serde(rename = "ID", borrow)]
    pub id: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct Configurations<'a> {
    #[serde(borrow)]
    pub nodes: Vec<Node<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct Node<'a> {
    #[serde(borrow)]
    pub operator: &'a str,
    #[serde(borrow)]
    pub cpe_match: Vec<CpeMatch<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct CpeMatch<'a> {
    pub vulnerable: bool,
    #[serde(rename = "cpe23Uri", borrow)]
    pub cpe: Cow<'a, str>,
    #[serde(rename = "versionStartIncluding", borrow)]
    pub version_start_including: Option<Cow<'a, str>>,
    #[serde(rename = "versionStartExcluding", borrow)]
    pub version_start_excluding: Option<Cow<'a, str>>,
    #[serde(rename = "versionEndIncluding", borrow)]
    pub version_end_including: Option<Cow<'a, str>>,
    #[serde(rename = "versionEndExcluding", borrow)]
    pub version_end_excluding: Option<Cow<'a, str>>,
}
