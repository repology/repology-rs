// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::net::IpAddr;
use std::sync::Arc;

use askama::Template;
use axum::Form;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use chrono::{DateTime, NaiveDate, Utc};
use indoc::indoc;
use serde::Deserialize;
use sqlx::FromRow;
use tower_cookies::{Cookie, Cookies};
use tracing::{error, info};

use crate::config::AppConfig;
use crate::endpoints::Endpoint;
use crate::extractors::PossibleClientAddresses;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

use super::common::Project;
use super::nonexistent::nonexisting_project;

#[derive(FromRow)]
struct Report {
    created: DateTime<Utc>,
    need_verignore: bool,
    need_split: bool,
    need_merge: bool,
    need_vuln: bool,
    comment: Option<String>,
    reply: Option<String>,
    accepted: Option<bool>,
}

#[derive(Deserialize, Default, Debug)]
pub struct ReportForm {
    #[serde(default, deserialize_with = "crate::query::deserialize_bool_flag")]
    need_verignore: bool,
    #[serde(default, deserialize_with = "crate::query::deserialize_bool_flag")]
    need_split: bool,
    #[serde(default, deserialize_with = "crate::query::deserialize_bool_flag")]
    need_merge: bool,
    #[serde(default, deserialize_with = "crate::query::deserialize_bool_flag")]
    need_vuln: bool,
    comment: String,
}

#[derive(Template)]
#[template(path = "project/report.html")]
struct TemplateParams {
    ctx: TemplateContext,
    project_name: String,
    project: Project,
    reports: Vec<Report>,
    reports_disabled: bool,
    too_many_reports: bool,
    afk_till: Option<NaiveDate>,
    form: ReportForm,
    errors: Vec<&'static str>,
    report_added_message: bool,
    redirect_from: Option<String>,
}

fn check_new_report(
    project_name: &str,
    project_is_alive: bool,
    too_many_reports: bool,
    client_addresses: &[IpAddr],
    form: &ReportForm,
    config: &AppConfig,
) -> std::result::Result<(), Vec<&'static str>> {
    const MAX_REPORT_COMMENT_LENGTH: usize = 10240;

    let mut errors: Vec<&str> = vec![];
    let mut is_spam = false;

    // sanity checks
    if !project_is_alive {
        error!("bad report: report to gone or nonexisting project");
        errors.push("project does not exist or is gone");
    }

    if too_many_reports {
        error!("bad report: too many reports for project");
        errors.push("too many reports for this project");
    }

    if form.comment.len() > MAX_REPORT_COMMENT_LENGTH {
        error!(
            comment_length = form.comment.len(),
            "bad report: comment too long"
        );
        errors.push("comment is too long");
    }

    if !form.need_verignore
        && !form.need_split
        && !form.need_merge
        && !form.need_vuln
        && form.comment.is_empty()
    {
        error!("bad report: report form is not filled");
        errors.push("please fill out the form");
    }

    if form.comment.contains("<a href") {
        error!("bad report: report comment contains HTML");
        errors.push("HTML not allowed");
    }

    if form.need_vuln && !form.comment.contains("nvd.nist.gov/vuln/detail/CVE-") {
        error!("bad report: missing vulnerability report does not contain NVD link");
        errors.push("link to NVD entry (e.g. https://nvd.nist.gov/vuln/detail/CVE-*) for missing CVE is required; note that CVE must already have CPE(s) assigned");
    }

    if config.disabled_reports.contains(project_name) {
        error!("bad report: report attempt to disabled project");
        errors.push("new reports to this project are disabled, probably due to a big number of incorrect reports or spam");
    }

    // spam checks
    for keyword in &config.spam_keywords {
        if form.comment.contains(keyword) {
            error!(keyword, "bad report: report comment contains spam keyword");
            is_spam = true;
            break;
        }
    }

    for spam_network in &config.spam_networks {
        if client_addresses
            .iter()
            .any(|address| spam_network.contains(*address))
        {
            error!(
                %spam_network, "bad report: report submitter is blacklisted"
            );
            is_spam = true;
            break;
        }
    }

    if form.need_verignore
        && form.need_split
        && form.need_merge
        && form.need_vuln
        && form.comment.is_empty()
    {
        error!("bad report: report form filled in meaningless pattern");
        is_spam = true;
    }

    if is_spam {
        errors.push("spammers not welcome");
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

async fn project_report_generic(
    project_name: String,
    state: &AppState,
    cookies: &Cookies,
    input: Option<(&[std::net::IpAddr], ReportForm)>,
) -> EndpointResult {
    let ctx = TemplateContext::new(
        Endpoint::ProjectReport,
        vec![("project_name".to_string(), project_name.clone())],
        vec![],
    );

    let redirect_from_cookie_name = format!("rdr_{}", project_name);
    let redirect_from = if let Some(cookie) = cookies.get(&redirect_from_cookie_name) {
        let value = cookie.value().to_string();
        cookies.remove(Cookie::build(redirect_from_cookie_name).path("/").into());
        Some(value)
    } else {
        None
    };

    let project: Option<Project> = sqlx::query_as(indoc! {"
        SELECT
            num_repos,
            has_cves,
            has_related,
            orphaned_at
        FROM metapackages
        WHERE effname = $1
    "})
    .bind(&project_name)
    .fetch_optional(&state.pool)
    .await?;

    let Some(project) = project else {
        return nonexisting_project(state, cookies, ctx, project_name, None).await;
    };

    let reports: Vec<Report> = sqlx::query_as(indoc! {"
        SELECT
            created,
            need_verignore,
            need_split,
            need_merge,
            need_vuln,
            comment,
            reply,
            accepted
        FROM reports
        WHERE effname = $1
        ORDER BY created DESC
    "})
    .bind(&project_name)
    .fetch_all(&state.pool)
    .await?;

    let too_many_reports = reports.len() >= crate::constants::MAX_REPORTS;
    let report_added_cookie_name = format!("rprt_{}", project_name);

    let errors = if let Some((client_addresses, form)) = &input {
        if let Err(errors) = check_new_report(
            &project_name,
            !project.is_orphaned(),
            too_many_reports,
            client_addresses,
            form,
            &state.config,
        ) {
            errors
        } else {
            sqlx::query(indoc! {"
                INSERT INTO reports (
                    effname,
                    need_verignore,
                    need_split,
                    need_merge,
                    need_vuln,
                    comment
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6
                );
            "})
            .bind(&project_name)
            .bind(form.need_verignore)
            .bind(form.need_split)
            .bind(form.need_merge)
            .bind(form.need_vuln)
            .bind(&form.comment)
            .execute(&state.pool)
            .await?;

            info!("new report added");

            cookies.add(
                Cookie::build(report_added_cookie_name)
                    .max_age(tower_cookies::cookie::time::Duration::seconds(60))
                    .into(),
            );

            return Ok((
                StatusCode::FOUND,
                [(
                    header::LOCATION,
                    HeaderValue::from_maybe_shared(ctx.url_for_self(&[("_fragment", "")])?)?,
                )],
            )
                .into_response());
        }
    } else {
        vec![]
    };

    if project.is_orphaned() && reports.is_empty() {
        return nonexisting_project(state, cookies, ctx, project_name, Some(project)).await;
    }

    let current_date = Utc::now().date_naive();
    let afk_till = state
        .config
        .staff_afk_periods
        .iter()
        .filter(|period| period.from <= current_date && current_date <= period.to)
        .map(|period| period.to)
        .next();

    let report_added_message = {
        let has_cookie = cookies.get(&report_added_cookie_name).is_some();
        if has_cookie {
            cookies.remove(report_added_cookie_name.into());
        }
        has_cookie
    };

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            project_name,
            project,
            reports,
            reports_disabled: false,
            too_many_reports,
            afk_till,
            form: input.map(|(_, form)| form).unwrap_or_default(),
            errors,
            report_added_message,
            redirect_from,
        }
        .render()?,
    )
        .into_response())
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all, fields(project_name = project_name, form = ?form, addresses = ?addresses)))]
pub async fn project_report_post(
    Path(project_name): Path<String>,
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    PossibleClientAddresses(addresses): PossibleClientAddresses,
    Form(form): Form<ReportForm>,
) -> EndpointResult {
    project_report_generic(project_name, &state, &cookies, Some((&addresses, form))).await
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip_all, fields(project_name = project_name)))]
pub async fn project_report_get(
    Path(project_name): Path<String>,
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    project_report_generic(project_name, &state, &cookies, None).await
}
