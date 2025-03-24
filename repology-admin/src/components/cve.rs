// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::{DateTime, Utc};
use maud::{Markup, html};
use sqlx::FromRow;

use crate::config::Config;

#[derive(FromRow)]
pub struct Cve {
    pub last_modified: DateTime<Utc>,
    pub cpe_vendor: String,
    pub cpe_product: String,
    pub cpe_edition: String,
    pub cpe_lang: String,
    pub cpe_sw_edition: String,
    pub cpe_target_sw: String,
    pub cpe_target_hw: String,
    pub cpe_other: String,
    pub cve_ids: Vec<String>,
    pub project_candidates: Vec<String>,
}

impl Cve {
    pub fn render(&self, config: &Config) -> Markup {
        html! {
            tr {
                td {
                    (self.last_modified.format("%Y-%m-%d %H:%M"))
                }
                td {
                    .field .is-grouped .is-grouped-multiline {
                        @if self.cpe_vendor != "*" {
                            .control {
                                .tags .has-addons {
                                    span .tag .is-rounded .is-dark { "vendor" }
                                    span .tag .is-rounded .is-info { (self.cpe_vendor) }
                                }
                            }
                        }
                        @if self.cpe_product != "*" {
                            .control {
                                .tags .has-addons {
                                    span .tag .is-rounded .is-dark { "product" }
                                    span .tag .is-rounded .is-info { (self.cpe_product) }
                                }
                            }
                        }
                        @if self.cpe_edition != "*" {
                            .control {
                                .tags .has-addons {
                                    span .tag .is-rounded .is-dark { "edition" }
                                    span .tag .is-rounded .is-info { (self.cpe_edition) }
                                }
                            }
                        }
                        @if self.cpe_lang != "*" {
                            .control {
                                .tags .has-addons {
                                    span .tag .is-rounded .is-dark { "lang" }
                                    span .tag .is-rounded .is-info { (self.cpe_lang) }
                                }
                            }
                        }
                        @if self.cpe_sw_edition != "*" {
                            .control {
                                .tags .has-addons {
                                    span .tag .is-rounded .is-dark { "sw_edition" }
                                    span .tag .is-rounded .is-info { (self.cpe_sw_edition) }
                                }
                            }
                        }
                        @if self.cpe_target_sw != "*" {
                            .control {
                                .tags .has-addons {
                                    span .tag .is-rounded .is-dark { "target_sw" }
                                    span .tag .is-rounded .is-info { (self.cpe_target_sw) }
                                }
                            }
                        }
                        @if self.cpe_target_hw != "*" {
                            .control {
                                .tags .has-addons {
                                    span .tag .is-rounded .is-dark { "target_hw" }
                                    span .tag .is-rounded .is-info { (self.cpe_target_hw) }
                                }
                            }
                        }
                        @if self.cpe_other != "*" {
                            .control {
                                .tags .has-addons {
                                    span .tag .is-rounded .is-dark { "other" }
                                    span .tag .is-rounded .is-info { (self.cpe_other) }
                                }
                            }
                        }
                    }
                }
                td {
                    .tags {
                        @for cve_id in self.cve_ids.iter() {
                            a .tag .is-small href={"https://nvd.nist.gov/vuln/detail/" (cve_id)} { (cve_id) };
                        }
                    }
                }
                td {
                    .field .is-grouped .is-grouped-multiline {
                        @for project_name in &self.project_candidates {
                            form .control hx-target="this" hx-swap="delete" hx-post="/parts/cpes" {
                                input type="hidden" name="effname" value=(project_name);
                                input type="hidden" name="cpe_vendor" value=(self.cpe_vendor);
                                input type="hidden" name="cpe_product" value=(self.cpe_product);
                                input type="hidden" name="cpe_edition" value=(self.cpe_edition);
                                input type="hidden" name="cpe_lang" value=(self.cpe_lang);
                                input type="hidden" name="cpe_sw_edition" value=(self.cpe_sw_edition);
                                input type="hidden" name="cpe_target_sw" value=(self.cpe_target_sw);
                                input type="hidden" name="cpe_target_hw" value=(self.cpe_target_hw);
                                input type="hidden" name="cpe_other" value=(self.cpe_other);
                                .tags .has-addons {
                                    a .tag href={(config.repology_host) "/project/" (project_name) "/versions"} {
                                        (project_name)
                                    }
                                    button .tag .is-success type="submit" {
                                        "add"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
