// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use indoc::indoc;
use maud::{Markup, html};
use sqlx::{FromRow, PgPool};

use crate::components::cpe::Cpe;
use crate::config::Config;

#[derive(FromRow)]
pub struct CpeItem {
    pub id: i32,
    #[sqlx(flatten)]
    pub cpe: Cpe,
    pub has_alive_project: bool,
    pub has_project_redirect: bool,
    pub has_cves: bool,
    pub has_dict: bool,
}

impl CpeItem {
    pub async fn fetch(pool: &PgPool, id: i32) -> Result<Self> {
        Ok(
            sqlx::query_as(indoc! {"
                SELECT
                    id,
                    effname,
                    cpe_vendor,
                    cpe_product,
                    cpe_edition,
                    cpe_lang,
                    cpe_sw_edition,
                    cpe_target_sw,
                    cpe_target_hw,
                    cpe_other,
                    EXISTS (
                        SELECT * FROM metapackages WHERE effname = manual_cpes.effname AND num_repos > 0
                    ) AS has_alive_project,
                    EXISTS (
                        SELECT * FROM project_redirects AS old INNER JOIN project_redirects AS new USING(repository_id, trackname)
                        WHERE NOT old.is_actual AND new.is_actual AND old.project_id = (SELECT id FROM metapackages WHERE effname = manual_cpes.effname)
                    ) AS has_project_redirect,
                    EXISTS (
                        SELECT * FROM vulnerable_cpes WHERE
                            cpe_vendor = manual_cpes.cpe_vendor AND
                            cpe_product = manual_cpes.cpe_product AND
                            coalesce(nullif(cpe_edition, '*') = nullif(manual_cpes.cpe_edition, '*'), TRUE) AND
                            coalesce(nullif(cpe_lang, '*') = nullif(manual_cpes.cpe_lang, '*'), TRUE) AND
                            coalesce(nullif(cpe_sw_edition, '*') = nullif(manual_cpes.cpe_sw_edition, '*'), TRUE) AND
                            coalesce(nullif(cpe_target_sw, '*') = nullif(manual_cpes.cpe_target_sw, '*'), TRUE) AND
                            coalesce(nullif(cpe_target_hw, '*') = nullif(manual_cpes.cpe_target_hw, '*'), TRUE) AND
                            coalesce(nullif(cpe_other, '*') = nullif(manual_cpes.cpe_other, '*'), TRUE)
                    ) AS has_cves,
                    EXISTS (
                        SELECT * FROM cpe_dictionary WHERE
                            cpe_vendor = manual_cpes.cpe_vendor AND
                            cpe_product = manual_cpes.cpe_product AND
                            coalesce(nullif(cpe_edition, '*') = nullif(manual_cpes.cpe_edition, '*'), TRUE) AND
                            coalesce(nullif(cpe_lang, '*') = nullif(manual_cpes.cpe_lang, '*'), TRUE) AND
                            coalesce(nullif(cpe_sw_edition, '*') = nullif(manual_cpes.cpe_sw_edition, '*'), TRUE) AND
                            coalesce(nullif(cpe_target_sw, '*') = nullif(manual_cpes.cpe_target_sw, '*'), TRUE) AND
                            coalesce(nullif(cpe_target_hw, '*') = nullif(manual_cpes.cpe_target_hw, '*'), TRUE) AND
                            coalesce(nullif(cpe_other, '*') = nullif(manual_cpes.cpe_other, '*'), TRUE)
                    ) AS has_dict
                FROM manual_cpes
                WHERE id = $1
            "})
            .bind(id)
            .fetch_one(pool)
            .await?
        )
    }

    pub async fn delete(pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM manual_cpes WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub fn render(&self, config: &Config) -> Markup {
        html! {
            tr .has-background-warning-light[!self.has_alive_project || !self.has_cves && !self.has_dict] {
                td {
                    a href={(config.repology_host) "/project/" (self.cpe.effname) "/versions"} {
                        (self.cpe.effname)
                    }
                }
                td {
                    (self.cpe.cpe_vendor)
                    ":"
                    (self.cpe.cpe_product)
                    ":"
                    (self.cpe.cpe_edition)
                    ":"
                    (self.cpe.cpe_lang)
                    ":"
                    (self.cpe.cpe_sw_edition)
                    ":"
                    (self.cpe.cpe_target_sw)
                    ":"
                    (self.cpe.cpe_target_hw)
                    ":"
                    (self.cpe.cpe_other)
                }
                td .has-text-centered {
                    @if self.has_alive_project {
                        .tag .is-success { "alive" }
                    } @else if self.has_project_redirect {
                        .tag .is-danger { "redirect" }
                    } @else {
                        .tag .is-warning { "missing" }
                    }
                }
                td .has-text-centered {
                    @if self.has_cves {
                        .tag .is-success { "yes" }
                    } @else {
                        .tag .is-warning { "no" }
                    }
                }
                td .has-text-centered {
                    @if self.has_dict {
                        .tag .is-success { "yes" }
                    } @else {
                        .tag .is-warning { "no" }
                    }
                }
                td .buttons .is-centered .has-addons .is-flex-wrap-nowrap {
                    button .button .is-small hx-get={"/parts/cpes/" (self.id) "/form"} hx-target="closest tr" hx-swap="outerHTML" { "Edit" }
                    button .button .is-small .is-danger hx-delete={"/parts/cpes/" (self.id)} hx-confirm={"Really delete CPE for " (self.cpe.effname) "?"} hx-swap="delete" hx-target="closest tr" { "Delete" }
                }
            }
        }
    }

    pub fn render_form(&self) -> Markup {
        html! {
            tr .has-background-warning-light[!self.has_alive_project || !self.has_cves && !self.has_dict] {
                td colspan="6" {
                    form hx-put={"/parts/cpes/" (self.id)} hx-target="closest tr" hx-swap="outerHTML" {
                        (self.cpe.render_form_fields(true))
                        .buttons {
                            button .button .is-success type="submit" { "Save" }
                            button .button hx-get={"/parts/cpes/" (self.id)} hx-target="closest tr" hx-swap="outerHTML" { "Cancel" }
                        }
                    }
                }
            }
        }
    }
}
