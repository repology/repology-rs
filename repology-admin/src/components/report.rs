// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use chrono::{DateTime, Utc};
use indoc::indoc;
use maud::{Markup, html};
use serde::Deserialize;
use sqlx::{FromRow, PgPool};

use crate::config::Config;

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ReportFlags {
    #[serde(default)]
    pub hide_processed: bool,
}

impl ReportFlags {
    pub fn as_query_string(&self) -> String {
        if self.hide_processed {
            "?hide_processed=true".into()
        } else {
            "".into()
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportUpdate {
    reply: String,
    // TODO: serialize from
    accepted: String,
}

#[derive(FromRow)]
pub struct Report {
    pub id: i32,
    pub effname: String,
    pub created: DateTime<Utc>,
    pub need_verignore: bool,
    pub need_split: bool,
    pub need_merge: bool,
    pub need_vuln: bool,
    pub comment: Option<String>,
    pub reply: Option<String>,
    pub accepted: Option<bool>,
}

impl Report {
    pub async fn fetch(pool: &PgPool, id: i32) -> Result<Self> {
        Ok(sqlx::query_as(indoc! {"
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
            WHERE id = $1
        "})
        .bind(id)
        .fetch_one(pool)
        .await?)
    }

    pub async fn update(pool: &PgPool, id: i32, update: &ReportUpdate) -> Result<()> {
        sqlx::query(indoc! {"
            UPDATE reports
            SET
                reply = $2,
                accepted = $3
            WHERE id = $1
        "})
        .bind(id)
        .bind(&update.reply)
        .bind(match update.accepted.as_ref() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        })
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM reports WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub fn render(&self, flags: &ReportFlags, config: &Config) -> Markup {
        html! {
            .card .has-background-success-light[self.accepted == Some(true)] .has-background-danger-light[self.accepted == Some(false)] hx-target="this" hx-swap="outerHTML" {
                form hx-patch={"/parts/reports/" (self.id) (flags.as_query_string())} {
                    .card-header {
                        .card-header-title {
                            .mr-2 {
                                a href={(config.repology_host) "/project/" (self.effname) "/versions"} { (self.effname) }
                                @if let Some(classified_effname) = self.effname.strip_suffix("-unclassified") {
                                    " (see also ";
                                    a href={(config.repology_host) "/project/" (classified_effname) "/versions"} { (classified_effname) };
                                    ")"
                                }
                            }
                            @if self.need_verignore || self.need_split || self.need_merge || self.need_vuln {
                                .tags {
                                    @if self.need_verignore { span .tag .is-light { "ignore" } }
                                    @if self.need_split { span .tag .is-light { "split" } }
                                    @if self.need_merge { span .tag .is-light { "merge" } }
                                    @if self.need_vuln { span .tag .is-light { "vuln" } }
                                }
                            }
                        }
                        .card-header-icon {
                            ( (Utc::now() - self.created).num_hours() ) "h ago"
                        }
                    }

                    .card-content {
                        @if let Some(comment) = &self.comment {
                            .box {
                                @for (n, line) in comment.lines().enumerate() {
                                    @if n > 0 {
                                        br;
                                    }
                                    (line)
                                }
                            }
                        }
                        .form-group {
                            textarea .textarea id="reply" name="reply" rows="2" placeholder="Reply" {
                                (self.reply.as_deref().unwrap_or_default())
                            }
                        }
                    }

                    .card-footer {
                        button .card-footer-item .has-text-success type="submit" name="accepted" value="true" { "Accept" }
                        button .card-footer-item .has-text-danger type="submit" name="accepted" value="false"{ "Reject" }
                        button .card-footer-item type="submit" name="accepted" value="undefined" { "Leave unprocessed" }
                        button .card-footer-item hx-delete={"/parts/report/" (self.id)} hx-confirm="Really delete this report?" hx-swap="delete" { "Delete" }
                    }
                }
            }
        }
    }
}
