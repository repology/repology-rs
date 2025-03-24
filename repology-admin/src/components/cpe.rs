// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use indoc::indoc;
use maud::{Markup, html};
use serde::Deserialize;
use sqlx::{FromRow, PgPool};

#[derive(Deserialize, FromRow)]
pub struct Cpe {
    pub effname: String,
    pub cpe_vendor: String,
    pub cpe_product: String,
    pub cpe_edition: String,
    pub cpe_lang: String,
    pub cpe_sw_edition: String,
    pub cpe_target_sw: String,
    pub cpe_target_hw: String,
    pub cpe_other: String,
}

impl Cpe {
    pub fn new_for_create() -> Self {
        Self {
            effname: "".to_string(),
            cpe_vendor: "".to_string(),
            cpe_product: "".to_string(),
            cpe_edition: "*".to_string(),
            cpe_lang: "*".to_string(),
            cpe_sw_edition: "*".to_string(),
            cpe_target_sw: "*".to_string(),
            cpe_target_hw: "*".to_string(),
            cpe_other: "*".to_string(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.effname.is_empty() && !self.cpe_vendor.is_empty() && !self.cpe_product.is_empty()
    }

    pub async fn create(&self, pool: &PgPool) -> Result<i32> {
        let id = sqlx::query_scalar(indoc! {"
            INSERT INTO manual_cpes (
                effname,
                cpe_vendor,
                cpe_product,
                cpe_edition,
                cpe_lang,
                cpe_sw_edition,
                cpe_target_sw,
                cpe_target_hw,
                cpe_other
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9
            )
            RETURNING id
        "})
        .bind(&self.effname)
        .bind(&self.cpe_vendor)
        .bind(&self.cpe_product)
        .bind(&self.cpe_edition)
        .bind(&self.cpe_lang)
        .bind(&self.cpe_sw_edition)
        .bind(&self.cpe_target_sw)
        .bind(&self.cpe_target_hw)
        .bind(&self.cpe_other)
        .fetch_one(pool)
        .await?;

        Ok(id)
    }

    pub async fn update(&self, pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query(indoc! {"
            UPDATE manual_cpes
            SET
                effname = $2,
                cpe_vendor = $3,
                cpe_product = $4,
                cpe_edition = $5,
                cpe_lang = $6,
                cpe_sw_edition = $7,
                cpe_target_sw = $8,
                cpe_target_hw = $9,
                cpe_other = $10
            WHERE id = $1
        "})
        .bind(id)
        .bind(&self.effname)
        .bind(&self.cpe_vendor)
        .bind(&self.cpe_product)
        .bind(&self.cpe_edition)
        .bind(&self.cpe_lang)
        .bind(&self.cpe_sw_edition)
        .bind(&self.cpe_target_sw)
        .bind(&self.cpe_target_hw)
        .bind(&self.cpe_other)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub fn render_form_fields(&self, validation: bool) -> Markup {
        html! {
            .is-flex .is-flex-wrap-wrap {
                .field {
                    @let is_invalid = validation && self.effname.is_empty();
                    label.label { "project" }
                    .control {
                        input .input .is-danger[is_invalid] type="text" name="effname" placeholder="Project name" value=(self.effname);
                    }
                    @if is_invalid {
                        p .help .is-danger {
                            "Project name cannot be empty"
                        }
                    }
                }
                .field {
                    @let is_invalid = validation && self.cpe_vendor.is_empty();
                    label.label { "vendor" }
                    .control {
                        input .input .is-danger[is_invalid] type="text" name="cpe_vendor" placeholder="CPE vendor" value=(self.cpe_vendor);
                    }
                    @if is_invalid {
                        p .help .is-danger {
                            "CPE vendor cannot be empty"
                        }
                    }
                }
                .field {
                    @let is_invalid = validation && self.cpe_product.is_empty();
                    label.label { "product" }
                    .control {
                        input .input .is-danger[is_invalid] type="text" name="cpe_product" placeholder="CPE product" value=(self.cpe_product);
                    }
                    @if is_invalid {
                        p .help .is-danger {
                            "CPE product cannot be empty"
                        }
                    }
                }
                .field {
                    label.label { "edition" }
                    .control {
                        input .input type="text" name="cpe_edition" placeholder="CPE edition" value=(self.cpe_edition);
                    }
                }
                .field {
                    label.label { "lang" }
                    .control {
                        input .input type="text" name="cpe_lang" placeholder="CPE lang" value=(self.cpe_lang);
                    }
                }
                .field {
                    label.label { "sw_edition" }
                    .control {
                        input .input type="text" name="cpe_sw_edition" placeholder="CPE sw_edition" value=(self.cpe_sw_edition);
                    }
                }
                .field {
                    label.label { "target_sw" }
                    .control {
                        input .input type="text" name="cpe_target_sw" placeholder="CPE target_sw" value=(self.cpe_target_sw);
                    }
                }
                .field {
                    label.label { "target_hw" }
                    .control {
                        input .input type="text" name="cpe_target_hw" placeholder="CPE target_hw" value=(self.cpe_target_hw);
                    }
                }
                .field {
                    label.label { "other" }
                    .control {
                        input .input type="text" name="cpe_other" placeholder="CPE other" value=(self.cpe_other);
                    }
                }
            }
        }
    }

    pub fn render_new_form(&self, validation: bool) -> Markup {
        html! {
            tr {
                td colspan="6" {
                    form hx-post="/parts/cpes" hx-target="closest tr" hx-swap="outerHTML" {
                        (self.render_form_fields(validation))
                        .buttons {
                            button .button .is-success type="submit" { "Save" }
                            // TODO: should be able to avoid HTTP request here
                            button .button hx-get="/parts/cpes/form" hx-target="closest tr" hx-swap="delete" { "Cancel" }
                        }
                    }
                }
            }
        }
    }
}
