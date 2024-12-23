// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use chrono::{DateTime, TimeDelta, Utc};
use indoc::indoc;
use libversion::version_compare;
use serde::Deserialize;
use sqlx::FromRow;

use crate::endpoints::Endpoint;
use crate::repository_data::RepositoriesDataSnapshot;
use crate::result::EndpointResult;
use crate::state::AppState;
use crate::template_context::TemplateContext;

use super::common::Project;
use super::nonexistent::nonexisting_project;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    #[serde(default)]
    #[serde(deserialize_with = "crate::query::deserialize_bool_flag")]
    pub autorefresh: bool,
}

#[derive(FromRow)]
struct RawEvent {
    pub timestamp: DateTime<Utc>,
    pub data: sqlx::types::Json<RawEventData>,
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "data")] // XXX: this is fragile; switch to explicit type handling sometime
enum RawEventData {
    #[serde(rename = "history_start")]
    HistoryStart {
        #[serde(default)]
        devel_repos: Vec<String>,
        #[serde(default)]
        newest_repos: Vec<String>,
        all_repos: Vec<String>,
        #[serde(default)]
        devel_versions: Vec<String>,
        #[serde(default)]
        newest_versions: Vec<String>,
    },
    #[serde(rename = "repos_update")]
    RepositoriesUpdate {
        repos_added: Vec<String>,
        repos_removed: Vec<String>,
    },
    #[serde(rename = "version_update")]
    VersionUpdate {
        #[serde(default)]
        versions: Vec<String>,
        #[serde(default)]
        repos: Vec<String>,
        branch: String,
        passed: Option<f64>,
    },
    #[serde(rename = "catch_up")]
    CatchUp {
        repos: Vec<String>,
        branch: String,
        lag: Option<f64>,
    },
    #[serde(rename = "history_end")]
    HistoryEnd { last_repos: Vec<String> },
}

#[derive(Debug)]
enum EventData {
    HistoryStart {
        actual_repos: Vec<String>,
        old_repos: Vec<String>,
        devel_versions: Vec<String>,
        newest_versions: Vec<String>,
    },
    RepositoriesUpdate {
        repos_added: Vec<String>,
        repos_removed: Vec<String>,
    },
    VersionUpdate {
        versions: Vec<String>,
        repos: Vec<String>,
        branch: String,
        passed: Option<TimeDelta>,
    },
    CatchUp {
        repos: Vec<String>,
        branch: String,
        lag: Option<TimeDelta>,
    },
    HistoryEnd {
        last_repos: Vec<String>,
    },
}

#[derive(Debug)]
struct Event {
    pub timestamp: DateTime<Utc>,
    pub data: EventData,
}

fn sort_repositories(repositories: &mut [String], repositories_data: &RepositoriesDataSnapshot) {
    repositories_data.sort_repository_names(repositories);
}

fn sort_versions(versions: &mut [String]) {
    // XXX: here we do not have extra information (such as flags) required for
    // proper version sorting. This should be fixed by using correct order when
    // generating history events
    versions.sort_by(|a, b| version_compare(a, b).reverse());
}

fn translate_raw_event(
    raw_event: RawEvent,
    repositories_data: &RepositoriesDataSnapshot,
) -> Option<Event> {
    let event_data = match raw_event.data.0 {
        RawEventData::HistoryStart {
            devel_repos,
            newest_repos,
            mut all_repos,
            mut devel_versions,
            mut newest_versions,
        } => {
            let actual_repos: HashSet<_> = devel_repos.into_iter().chain(newest_repos).collect();
            all_repos.retain(|repo| !actual_repos.contains(repo));
            let mut actual_repos: Vec<_> = actual_repos.into_iter().collect();
            sort_repositories(&mut actual_repos, repositories_data);
            sort_repositories(&mut all_repos, repositories_data);
            sort_versions(&mut devel_versions);
            sort_versions(&mut newest_versions);
            Some(EventData::HistoryStart {
                actual_repos,
                old_repos: all_repos,
                devel_versions,
                newest_versions,
            })
        }
        RawEventData::RepositoriesUpdate {
            mut repos_added,
            mut repos_removed,
            ..
        } => {
            if repos_added.is_empty() && repos_removed.is_empty() {
                None
            } else {
                sort_repositories(&mut repos_added, repositories_data);
                sort_repositories(&mut repos_removed, repositories_data);
                Some(EventData::RepositoriesUpdate {
                    repos_added,
                    repos_removed,
                })
            }
        }
        RawEventData::VersionUpdate {
            mut versions,
            mut repos,
            branch,
            passed,
        } => {
            sort_repositories(&mut repos, repositories_data);
            sort_versions(&mut versions);
            Some(EventData::VersionUpdate {
                repos,
                versions,
                branch,
                passed: passed.map(|secs| TimeDelta::seconds(secs as i64)),
            })
        }
        RawEventData::CatchUp {
            mut repos,
            branch,
            lag,
        } => {
            if repos.is_empty() {
                None
            } else {
                sort_repositories(&mut repos, repositories_data);
                Some(EventData::CatchUp {
                    repos,
                    branch,
                    lag: lag.map(|secs| TimeDelta::seconds(secs as i64)),
                })
            }
        }
        RawEventData::HistoryEnd { mut last_repos } => {
            sort_repositories(&mut last_repos, repositories_data);
            Some(EventData::HistoryEnd { last_repos })
        }
    };
    event_data.map(|event_data| Event {
        timestamp: raw_event.timestamp,
        data: event_data,
    })
}

#[derive(Template)]
#[template(path = "project/history.html")]
struct TemplateParams<'a> {
    ctx: TemplateContext,
    project_name: String,
    project: Project,
    events: Vec<Event>,
    repositories_data: &'a RepositoriesDataSnapshot,
    autorefresh: bool,
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn project_history(
    Path(gen_path): Path<Vec<(String, String)>>,
    Query(gen_query): Query<Vec<(String, String)>>,
    Path(project_name): Path<String>,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let ctx = TemplateContext::new(Endpoint::ProjectHistory, gen_path, gen_query);

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
        return nonexisting_project(&state, ctx, project_name, None).await;
    };

    let events: Vec<RawEvent> = sqlx::query_as(indoc! {"
        SELECT
            ts AS timestamp,
            jsonb_build_object('type', type::text, 'data', data) AS data
        FROM metapackages_events
        WHERE effname = $1
        ORDER BY ts DESC, type DESC
        LIMIT $2
    "})
    .bind(&project_name)
    .bind(crate::constants::HISTORY_PER_PAGE as i32)
    .fetch_all(&state.pool)
    .await?;

    if project.is_orphaned() && events.is_empty() {
        return nonexisting_project(&state, ctx, project_name, Some(project)).await;
    }

    let repositories_data = state.repository_data_cache.snapshot();

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        )],
        TemplateParams {
            ctx,
            project_name,
            project,
            events: events
                .into_iter()
                .filter_map(|event| translate_raw_event(event, &repositories_data))
                .collect(),
            repositories_data: &repositories_data,
            autorefresh: query.autorefresh,
        }
        .render()?,
    )
        .into_response())
}
