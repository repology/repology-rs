// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use repology_common::{LinkType, PackageFlags, PackageStatus};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Link {
    pub r#type: LinkType,
    pub url: String,
    pub fragment: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExtraField {
    OneValue(String),
    ManyValues(Vec<String>),
}

#[derive(PartialEq, Eq)]
pub struct ParsedPackage {
    pub subrepo: Option<String>,

    pub srcname: Option<String>,
    pub binname: Option<String>,
    pub binnames: Vec<String>,
    pub trackname: Option<String>,
    pub visiblename: String,
    pub projectname_seed: String,

    pub rawversion: String,

    pub arch: Option<String>,

    pub maintainers: Vec<String>,
    pub category: Option<String>,
    pub comment: Option<String>,
    pub licenses: Vec<String>,

    pub extrafields: HashMap<String, ExtraField>,

    pub cpe_vendor: Option<String>,
    pub cpe_product: Option<String>,
    pub cpe_edition: Option<String>,
    pub cpe_lang: Option<String>,
    pub cpe_sw_edition: Option<String>,
    pub cpe_target_sw: Option<String>,
    pub cpe_target_hw: Option<String>,
    pub cpe_other: Option<String>,

    pub links: Vec<Link>,

    pub version: String,

    pub flags: PackageFlags,
    pub flavors: Vec<String>,
}

macro_rules! field {
    ($formatter:ident, $package:ident.$field:ident) => {
        $formatter.field(stringify!($field), &$package.$field);
    };
}

macro_rules! field_opt {
    ($formatter:ident, $package:ident.$field:ident) => {
        if let Some(value) = &$package.$field {
            $formatter.field(stringify!($field), value);
        }
    };
}

macro_rules! field_vec {
    ($formatter:ident, $package:ident.$field:ident) => {
        if !$package.$field.is_empty() {
            $formatter.field(stringify!($field), &$package.$field);
        }
    };
}

impl std::fmt::Debug for ParsedPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut f = f.debug_struct("Package");
        field_opt!(f, self.subrepo);
        field_opt!(f, self.srcname);
        field_opt!(f, self.binname);
        field_vec!(f, self.binnames);
        field_opt!(f, self.trackname);
        field!(f, self.visiblename);
        field!(f, self.projectname_seed);

        field!(f, self.rawversion);

        field_opt!(f, self.arch);

        field_vec!(f, self.maintainers);
        field_opt!(f, self.category);
        field_opt!(f, self.comment);
        field_vec!(f, self.licenses);

        field_vec!(f, self.extrafields);

        field_opt!(f, self.cpe_vendor);
        field_opt!(f, self.cpe_product);
        field_opt!(f, self.cpe_edition);
        field_opt!(f, self.cpe_lang);
        field_opt!(f, self.cpe_sw_edition);
        field_opt!(f, self.cpe_target_sw);
        field_opt!(f, self.cpe_target_hw);
        field_opt!(f, self.cpe_other);

        field_vec!(f, self.links);

        field!(f, self.version);

        if !self.flags.is_empty() {
            f.field("flags", &self.flags);
        }

        field_vec!(f, self.flavors);
        f.finish()
    }
}

#[expect(unused)] // sample of complete package, will be removed after more Package bits are added
#[derive(Debug, PartialEq, Eq)]
pub struct Package {
    pub repo: String,
    pub family: String,
    pub subrepo: Option<String>,

    pub srcname: Option<String>,
    pub binname: Option<String>,
    pub binnames: Vec<String>,
    pub trackname: Option<String>,
    pub visiblename: String,
    pub projectname_seed: String,

    pub origversion: String,
    pub rawversion: String,

    pub arch: Option<String>,

    pub maintainers: Vec<String>,
    pub category: Option<String>,
    pub comment: Option<String>,
    pub licenses: Vec<String>,

    pub extrafields: HashMap<String, String>,

    pub cpe_vendor: Option<String>,
    pub cpe_product: Option<String>,
    pub cpe_edition: Option<String>,
    pub cpe_lang: Option<String>,
    pub cpe_sw_edition: Option<String>,
    pub cpe_target_sw: Option<String>,
    pub cpe_target_hw: Option<String>,
    pub cpe_other: Option<String>,

    pub links: Vec<Link>,

    pub effname: String,

    pub version: String,
    pub versionclass: PackageStatus,

    pub flags: PackageFlags,
    pub shadow: bool,
    pub flavors: Vec<String>,
    pub branch: Option<String>,
}
