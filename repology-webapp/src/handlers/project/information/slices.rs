// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;
use std::collections::HashMap;

use crate::package::summarization::DisplayVersion;
use crate::repository_data::RepositoryData;

use super::Link;

#[derive(PartialEq, Eq, Hash)]
pub enum StringSliceType {
    Name,
    Summary,
    Maintainer,
    Category,
    License,
}

#[derive(PartialEq, Eq, Hash)]
pub enum LinkSliceType {
    Homepage,
    Download,
    Issues,
    Repository,
    Documentation,
    Recipe,
    Package,
    Patch,
    BuildLog,
}

pub struct StringSliceItem<'a> {
    pub value: &'a str,
    pub spread: usize,
}

pub struct LinkSliceItem<'a> {
    pub url: Cow<'a, str>,
    pub spread: usize,
    pub link: &'a Link,
}

pub struct Slices<'a> {
    // XXX: can use non-owninig DisplayVersion here
    pub versions: Vec<DisplayVersion>,
    pub repositories: Vec<&'a RepositoryData>,
    pub string_slices: HashMap<StringSliceType, Vec<StringSliceItem<'a>>>,
    pub link_slices: HashMap<LinkSliceType, Vec<LinkSliceItem<'a>>>,
    pub maintainer_emails: Option<String>,
}
