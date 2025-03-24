// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use maud::{DOCTYPE, Markup, html};

pub fn render_html(inner_content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title{ "Repology admin" };
                link href="/bulma.min.css" rel="stylesheet";
                link href="/repology-admin.css" rel="stylesheet";
            }
            body {
                #wrapper {
                    nav .navbar role="navigation" aria-label="main navigation" {
                        .container {
                            .navbar-menu .is-active {
                                .navbar-start {
                                    a .navbar-item hx-get="/reports/new" hx-target="#content" hx-swap="innerHTML" hx-push-url="true" { "New reports" }
                                    a .navbar-item hx-get="/reports/all" hx-target="#content" hx-swap="innerHTML" hx-push-url="true" { "All reports" }
                                    a .navbar-item hx-get="/cpes" hx-target="#content" hx-swap="innerHTML" hx-push-url="true" { "CPEs" }
                                    a .navbar-item hx-get="/cves" hx-target="#content" hx-swap="innerHTML" hx-push-url="true" { "CVEs" }
                                }
                            }
                        }
                    }
                    section .section {
                        .container #content {
                            (inner_content)
                        }
                    }
                }
                footer .footer {
                    .container {
                        p {
                            "Copyright (C) 2016-2025 Dmitry Marakasov";
                            br;
                            "Code licensed under GPLv3+."
                        }
                    }
                }
                script src="/htmx.min.js" {}
            }
        }
    }
}
