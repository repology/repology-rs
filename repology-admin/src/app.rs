// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::config::Config;
use crate::handlers;

pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}

pub fn create_app(pool: PgPool, config: Config) -> Result<Router> {
    use handlers::cpes::*;
    use handlers::cves::*;
    use handlers::index::*;
    use handlers::reports::*;
    use handlers::r#static::*;
    let router = Router::new()
        // static files
        .route("/htmx.min.js", get(handle_htmx))
        .route("/bulma.min.css", get(handle_bulma))
        .route("/repology-admin.css", get(handle_css))
        // redirect to default page
        .route("/", get(handle_index))
        // reports
        .route("/reports/new", get(handle_new_reports_page))
        .route("/reports/all", get(handle_all_reports_page))
        // cpes
        .route("/cpes", get(handle_cpes_page))
        .route("/parts/cpes", get(handle_cpes))
        .route("/parts/cpes/form", get(handle_new_cpe_form))
        .route("/parts/cpes/{id}", get(handle_cpe))
        .route("/parts/cpes/{id}/form", get(handle_cpe_form))
        // cves
        .route("/cves", get(handle_cves_page))
        .route("/parts/cves", get(handle_cves));

    let router = if config.allow_changes {
        use axum::routing::{delete, patch, post, put};
        router
            // reports
            .route("/parts/reports/{id}", delete(handle_delete_report))
            .route("/parts/reports/{id}", patch(handle_patch_report))
            // cpes
            .route("/parts/cpes", post(handle_new_cpe))
            .route("/parts/cpes/{id}", delete(handle_delete_cpe))
            .route("/parts/cpes/{id}", put(handle_update_cpe))
    } else {
        router
    };

    let state = Arc::new(AppState { pool, config });

    let router = router.with_state(state);

    Ok(router)
}
