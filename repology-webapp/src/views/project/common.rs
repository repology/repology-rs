// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(FromRow)]
pub struct Project {
    #[sqlx(try_from = "i16")]
    pub num_repos: u32,
    pub has_cves: bool,
    pub has_related: bool,
    pub orphaned_at: Option<DateTime<Utc>>,
}
