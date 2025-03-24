// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use indoc::indoc;
use maud::{Markup, html};
use sqlx::PgPool;

use crate::components::report::{Report, ReportFlags};
use crate::config::Config;

pub struct Reports {
    reports: Vec<Report>,
}

impl Reports {
    pub async fn fetch_new(pool: &PgPool) -> Result<Self> {
        let reports: Vec<Report> = sqlx::query_as(indoc! {"
            SELECT
                id,
                effname,
                created,
                need_verignore,
                need_split,
                need_merge,
                need_vuln,
                comment,
                reply,
                accepted
            FROM reports
            WHERE accepted IS NULL
            ORDER BY id DESC
            LIMIT 100
        "})
        .fetch_all(pool)
        .await?;

        Ok(Self { reports })
    }

    pub async fn fetch_all(pool: &PgPool) -> Result<Self> {
        let reports: Vec<Report> = sqlx::query_as(indoc! {"
            SELECT
                id,
                effname,
                created,
                need_verignore,
                need_split,
                need_merge,
                need_vuln,
                comment,
                reply,
                accepted
            FROM reports
            ORDER BY id DESC
            LIMIT 100
        "})
        .fetch_all(pool)
        .await?;

        Ok(Self { reports })
    }

    pub fn render(&self, flags: &ReportFlags, config: &Config) -> Markup {
        html! {
            @for report in &self.reports {
                (report.render(flags, config))
            }
        }
    }
}
