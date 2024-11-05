// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

// TODO: make this configurable
pub const REPOLOGY_HOSTNAME: &str = "https://repology.org";

#[expect(unused)]
pub const CVES_PER_PAGE: u32 = 200;
#[expect(unused)]
pub const HISTORY_PER_PAGE: u32 = 500;
#[expect(unused)]
pub const MAINTAINERS_PER_PAGE: u32 = 200;
pub const PROBLEMS_PER_PAGE: u32 = 200;
pub const PROJECTS_PER_PAGE: u32 = 200;
pub const REDIRECTS_PER_PAGE: u32 = 200;
#[expect(unused)]
pub const REPORTS_PER_PAGE: u32 = 100;
#[expect(unused)]
pub const TRENDING_PER_PAGE: u32 = 25;
#[expect(unused)]
pub const TURNOVER_PER_PAGE: u32 = 350;
pub const HTML_FEED_MAX_ENTRIES: u32 = 500;
pub const ATOM_FEED_MAX_ENTRIES: u32 = 500;
pub const ATOM_FEED_MAX_AGE: Duration = Duration::from_days(31);
pub const ATOM_FEED_MIN_ENTRIES: u32 = 1;
