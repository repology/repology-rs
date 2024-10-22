// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use serde_repr::Deserialize_repr;

#[derive(Debug, PartialEq, Eq, sqlx::Type, PartialOrd, Ord, Hash, Deserialize_repr)]
#[repr(i8)]
pub enum LinkType {
    UpstreamHomepage = 0,
    UpstreamDownload = 1,
    UpstreamRepository = 2,
    UpstreamIssueTracker = 3,
    ProjectHomepage = 4,
    PackageHomepage = 5,
    PackageDownload = 6,
    PackageSources = 7,
    PackageIssueTracker = 8,
    PackageRecipe = 9,
    PackageRecipeRaw = 10,
    PackagePatch = 11,
    PackagePatchRaw = 12,
    PackageBuildLog = 13,
    PackageBuildLogRaw = 14,
    PackageNewVersionChecker = 15,
    UpstreamDocumentation = 16,
    UpstreamChangelog = 17,
    ProjectDownload = 18,
    UpstreamDonation = 19, // XXX: to be used sparingly not to provide obsolete funding info
    UpstreamDiscussion = 20,
    UpstreamCoverage = 21,
    UpstreamCi = 22,
    UpstreamWiki = 23,
    PackageStatistics = 25,
    PackageBuildStatus = 26,
    PackageBuildLogs = 27,
    UpstreamDownloadPage = 28,
    Other = 99,
}

impl LinkType {
    pub fn is_raw(&self) -> bool {
        use LinkType::*;
        match self {
            PackageRecipeRaw | PackagePatchRaw | PackageBuildLogRaw => true,
            _ => false,
        }
    }

    pub fn raw_counterpart(&self) -> Option<Self> {
        use LinkType::*;
        match self {
            PackageBuildLog => Some(PackageBuildLogRaw),
            PackagePatch => Some(PackagePatchRaw),
            PackageRecipe => Some(PackageRecipeRaw),
            _ => None,
        }
    }
}
