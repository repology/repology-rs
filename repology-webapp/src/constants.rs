// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

// TODO: make this configurable
pub const REPOLOGY_HOSTNAME: &str = "https://repology.org";

pub const BUILD_INFO: Option<&str> = option_env!("REPOLOGY_BUILD_INFO");

pub const CVES_PER_PAGE: usize = 200;
pub const HISTORY_PER_PAGE: usize = 500;
pub const MAINTAINERS_PER_PAGE: usize = 200;
pub const PROBLEMS_PER_PAGE: usize = 200;
pub const API_PROBLEMS_PER_PAGE: usize = 200;
pub const PROJECTS_PER_PAGE: usize = 200;
pub const REDIRECTS_PER_PAGE: usize = 200;
pub const MAX_REPORTS: usize = 100;
pub const TRENDING_PROJECTS_PERIOD: Duration = Duration::from_days(31);
pub const DECLINING_PROJECTS_PERIOD: Duration = Duration::from_days(91);
pub const MAX_TRENDING_PROJECTS: usize = 25;
#[expect(unused)]
pub const TURNOVER_PER_PAGE: usize = 350;
pub const HTML_FEED_MAX_ENTRIES: usize = 500;
pub const ATOM_FEED_MAX_ENTRIES: usize = 500;
pub const ATOM_FEED_MAX_AGE: Duration = Duration::from_days(31);
pub const ATOM_FEED_MIN_ENTRIES: usize = 1;
pub const NUM_SIMILAR_MAINTAINERS: usize = 50;
pub const MAX_MAINTAINER_PROJECTS: usize = 500;
pub const REPOSITORY_CACHE_REFRESH_PERIOD: Duration = Duration::from_mins(5);
pub const IMPORTANT_PROJECTS_CACHE_REFRESH_PERIOD: Duration = Duration::from_mins(10);
pub const RECENT_CPES_MAX_COUNT: usize = 200;
pub const RECENT_CVES_MAX_COUNT: usize = 200;
pub const RECENT_CVES_MAX_AGE: Duration = Duration::from_days(31);
pub const REPOSITORY_TOP_SIZE: usize = 10;
pub const MIN_REPOSITORY_SIZE_FOR_TOP: usize = 1000;
pub const IMPORTANT_UPDATES_AGE: Duration = Duration::from_days(1);
pub const IMPORTANT_UPDATES_MAX_COUNT: usize = 100;
