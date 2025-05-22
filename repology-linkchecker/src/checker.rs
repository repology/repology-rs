// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Instant;

use chrono::{DateTime, Utc};
use http::status::StatusCode;
use metrics::{counter, histogram};
use tracing::{error, info, warn};
use url::Url;

use repology_common::{LinkStatus, LinkStatusWithRedirect};

use crate::delayer::Delayer;
use crate::errors::extract_status;
use crate::hosts::{Hosts, RecheckCase};
use crate::http_client::{HttpClient, HttpMethod, HttpRequest, HttpResponse};
use crate::resolver::{IpVersion, Resolver, ResolverCache};
use crate::updater::CheckResult;

const MAX_REDIRECTS: usize = 10;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CheckPriority {
    Manual,
    Generated,
}

impl CheckPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "Manual",
            Self::Generated => "Generated",
        }
    }
}

pub struct CheckTask {
    pub id: i32,
    pub url: String,
    pub priority: CheckPriority,
    pub last_checked: Option<DateTime<Utc>>,
    pub deadline: DateTime<Utc>,
    pub prev_ipv4_status: LinkStatus,
    pub prev_ipv6_status: LinkStatus,
    pub last_success: Option<DateTime<Utc>>,
    // TODO[#260]: use for faster recheck of just-failed links, but first we need
    // to accumulate some failures #279 not to consider them just-failed
    pub failure_streak: Option<u16>,
}

pub struct Checker<'a, R, ER> {
    resolver_cache_ipv4: ResolverCache,
    resolver_cache_ipv6: ResolverCache,
    hosts: &'a Hosts,
    delayer: &'a Delayer,
    http_client: &'a R,
    experimental_http_client: &'a ER,
    disable_ipv4: bool,
    disable_ipv6: bool,
    satisfy_with_ipv6: bool,
}

impl<'a, R, ER> Checker<'a, R, ER>
where
    R: HttpClient + Send + Sync + 'static,
    ER: HttpClient + Send + Sync + 'static,
{
    pub fn new(
        resolver: &Resolver,
        hosts: &'a Hosts,
        delayer: &'a Delayer,
        http_client: &'a R,
        experimental_http_client: &'a ER,
    ) -> Self {
        Self {
            resolver_cache_ipv4: resolver.create_cache(IpVersion::Ipv4),
            resolver_cache_ipv6: resolver.create_cache(IpVersion::Ipv6),
            hosts,
            delayer,
            http_client,
            experimental_http_client,
            disable_ipv4: false,
            disable_ipv6: false,
            satisfy_with_ipv6: false,
        }
    }

    pub fn with_disable_ipv4(mut self, disable_ipv4: bool) -> Self {
        self.disable_ipv4 = disable_ipv4;
        self
    }

    pub fn with_disable_ipv6(mut self, disable_ipv6: bool) -> Self {
        self.disable_ipv6 = disable_ipv6;
        self
    }

    pub fn with_satisfy_with_ipv6(mut self, satisfy_with_ipv6: bool) -> Self {
        self.satisfy_with_ipv6 = satisfy_with_ipv6;
        self
    }

    async fn perform_http_request(
        host: &str,
        hosts: &Hosts,
        delayer: &Delayer,
        request: HttpRequest,
        http_client: &R,
        experimental_http_client: &ER,
    ) -> HttpResponse {
        let host_settings = hosts.get_settings(host);
        delayer
            .reserve(hosts.get_aggregation(host), host_settings.delay)
            .await;
        counter!("repology_linkchecker_checker_http_requests_total", "method" => request.method.as_str()).increment(1);
        let response = http_client.request(request.clone()).await;

        let experiment_prob: f32 = match response.status {
            LinkStatus::Http(429) => 0.0,
            LinkStatus::Http(200) => 0.001,
            LinkStatus::Http(403)
            | LinkStatus::Http(404)
            | LinkStatus::Http(500)
            | LinkStatus::Timeout => 0.01,
            _ => 1.0,
        };

        if rand::random::<f32>() < experiment_prob {
            delayer
                .reserve(hosts.get_aggregation(host), host_settings.delay)
                .await;
            counter!("repology_linkchecker_checker_http_requests_total", "method" => request.method.as_str()).increment(1);
            let experimental_response = experimental_http_client.request(request.clone()).await;

            let ignore_experiment =
                host == "code.google.com" // flapping 500's
                || host == "pyropus.ca." // native checker is correct
                || response.status == LinkStatus::Http(429) || experimental_response.status == LinkStatus::Http(429) // 429s
                || experimental_response.status == LinkStatus::Http(200) // flapping
                || experimental_response.status == LinkStatus::Timeout   // flapping
                || host == "packages.debian.org" || host == "packages.trisquel.org" || host == "packages.ubuntu.com" // flapping 502/504s
            ;

            if ignore_experiment {
            } else if response.status != experimental_response.status {
                counter!("repology_linkchecker_checker_experimental_requests_total", "outcome" => "mismatch", "status" => response.status.to_string()).increment(1);
                error!(url = request.url, status = ?response.status, experimental_status = ?experimental_response.status, "experimental status mismatch");
            } else {
                counter!("repology_linkchecker_checker_experimental_requests_total", "outcome" => "match")
                    .increment(1);
            }
        }

        response
    }

    async fn handle_one_request(
        url: &Url,
        hosts: &Hosts,
        delayer: &Delayer,
        resolver_cache: &mut ResolverCache,
        http_client: &R,
        experimental_http_client: &ER,
    ) -> Result<HttpResponse, LinkStatus> {
        let host = url
            .host_str()
            .expect("only urls with host should end up here");

        match resolver_cache.lookup(host).await {
            Ok(address) => {
                if !address.is_global() {
                    return Err(LinkStatus::DnsIpv4MappedInAaaa);
                }

                let host_settings = hosts.get_settings(host);

                if !host_settings.disable_head {
                    let head_response = Self::perform_http_request(
                        host,
                        hosts,
                        delayer,
                        HttpRequest {
                            url: url.as_str().to_string(),
                            method: HttpMethod::Head,
                            address,
                            timeout: host_settings.timeout,
                        },
                        http_client,
                        experimental_http_client,
                    )
                    .await;

                    if head_response.status
                        != LinkStatus::Http(StatusCode::METHOD_NOT_ALLOWED.as_u16())
                    {
                        return Ok(head_response);
                    }
                }

                Ok(Self::perform_http_request(
                    host,
                    hosts,
                    delayer,
                    HttpRequest {
                        url: url.as_str().to_string(),
                        method: HttpMethod::Get,
                        address,
                        timeout: host_settings.timeout,
                    },
                    http_client,
                    experimental_http_client,
                )
                .await)
            }
            Err(resolve_error) => Err(extract_status(&resolve_error, url.as_str())),
        }
    }

    async fn handle_one_check(
        url: Url,
        hosts: &Hosts,
        delayer: &Delayer,
        resolver_cache: &mut ResolverCache,
        http_client: &R,
        experimental_http_client: &ER,
    ) -> LinkStatusWithRedirect {
        let mut url = url;
        let mut num_redirects = 0;
        let mut had_temporary_redirect = false;
        let mut permanent_redirect_target: Option<String> = None;

        loop {
            let response = match Self::handle_one_request(
                &url,
                hosts,
                delayer,
                resolver_cache,
                http_client,
                experimental_http_client,
            )
            .await
            {
                Ok(response) => response,
                Err(status) => {
                    return status.into();
                }
            };

            let (status, location) = (response.status, response.location);

            if let Some(location) = location.filter(|_| status.is_redirect()) {
                let Ok(target) = url.join(&location) else {
                    error!(
                        url = url.as_str(),
                        location, "failed to join redirect target"
                    );
                    return LinkStatus::InvalidUrl.into();
                };

                if status.is_permanent_redirect() {
                    if !had_temporary_redirect {
                        permanent_redirect_target = Some(target.as_str().to_string());
                    }
                } else {
                    had_temporary_redirect = true;
                }
                url = target;
            } else {
                return LinkStatusWithRedirect {
                    status,
                    // only save redirects for successes
                    redirect: permanent_redirect_target
                        .filter(|_| status.is_success() == Some(true)),
                };
            }
            num_redirects += 1;
            if num_redirects >= MAX_REDIRECTS {
                return LinkStatus::TooManyRedirects.into();
            }
        }
    }

    pub async fn check(&mut self, task: CheckTask) -> CheckResult {
        let check_start = Instant::now();
        let mut host_settings = self.hosts.get_default_settings();
        let mut recheck_case: RecheckCase = task.priority.into();

        let ipv4_status: LinkStatusWithRedirect;
        let ipv6_status: LinkStatusWithRedirect;

        histogram!("repology_linkchecker_checker_task_overdue_age_seconds")
            .record((Utc::now() - task.deadline).as_seconds_f64());

        if let Some(url) = Url::parse(&task.url).ok().filter(|url| url.has_host()) {
            host_settings = self.hosts.get_settings(
                url.host_str()
                    .expect("only urls with host should end up here"),
            );

            if url.scheme() != "http" && url.scheme() != "https" {
                // do nothing; check result will be empty

                // Note: there's a bunch of other schemes in the wild, mainly for archaic
                // and made up protocols. We don't want to support these right now.
                //
                //   scheme   |  count
                // -----------+----------
                //  https     | 12604585
                //  http      |   339236
                //  ftp       |    17847
                //  mirror    |    15817
                //  mirrors   |     1535
                //  git+https |      381
                //  git       |      251
                //  svn+http  |       51
                //  svn       |       45
                //  cvs       |       27
                //  gopher    |       12
                //  bzr       |        4
                //  svn+https |        3
                //  hg        |        2
                //  hg+http   |        1
                //  git+http  |        1
                //  irc       |        1
                ipv4_status = if self.disable_ipv4 {
                    LinkStatus::ProtocolDisabled.into()
                } else {
                    LinkStatus::UnsupportedScheme.into()
                };
                ipv6_status = if self.disable_ipv6 {
                    LinkStatus::ProtocolDisabled.into()
                } else {
                    LinkStatus::UnsupportedScheme.into()
                };
                counter!("repology_linkchecker_checker_processed_total", "priority" => task.priority.as_str(), "class" => "InvalidScheme").increment(1);
            } else if host_settings.skip {
                ipv4_status = if self.disable_ipv4 {
                    LinkStatus::ProtocolDisabled.into()
                } else {
                    LinkStatus::Skipped.into()
                };
                ipv6_status = if self.disable_ipv6 {
                    LinkStatus::ProtocolDisabled.into()
                } else {
                    LinkStatus::Skipped.into()
                };
                counter!("repology_linkchecker_checker_processed_total", "priority" => task.priority.as_str(), "class" => "Skipped").increment(1);
            } else if task.priority == CheckPriority::Generated
                && task.id % 100 >= host_settings.generated_sampling_percentage as i32
            {
                ipv4_status = if self.disable_ipv4 {
                    LinkStatus::ProtocolDisabled.into()
                } else {
                    LinkStatus::OutOfSample.into()
                };
                ipv6_status = if self.disable_ipv6 {
                    LinkStatus::ProtocolDisabled.into()
                } else {
                    LinkStatus::OutOfSample.into()
                };
                recheck_case = RecheckCase::Unsampled;
                counter!("repology_linkchecker_checker_processed_total", "priority" => task.priority.as_str(), "class" => "Unsampled").increment(1);
            } else if host_settings.blacklist {
                ipv4_status = if self.disable_ipv4 {
                    LinkStatus::ProtocolDisabled.into()
                } else {
                    LinkStatus::Blacklisted.into()
                };
                ipv6_status = if self.disable_ipv6 {
                    LinkStatus::ProtocolDisabled.into()
                } else {
                    LinkStatus::Blacklisted.into()
                };
            } else {
                let mut skip_ipv4 = false;

                if self.disable_ipv6 {
                    ipv6_status = LinkStatus::ProtocolDisabled.into();
                } else if host_settings.disable_ipv6 {
                    ipv6_status = LinkStatus::ProtocolDisabledForHost.into();
                } else {
                    ipv6_status = Self::handle_one_check(
                        url.clone(),
                        self.hosts,
                        self.delayer,
                        &mut self.resolver_cache_ipv6,
                        self.http_client,
                        self.experimental_http_client,
                    )
                    .await;
                    skip_ipv4 = self.satisfy_with_ipv6 && ipv6_status.is_success() == Some(true);
                }

                if self.disable_ipv4 {
                    ipv4_status = LinkStatus::ProtocolDisabled.into();
                } else if skip_ipv4 {
                    ipv4_status = LinkStatus::SatisfiedWithIpv6Success.into();
                } else {
                    ipv4_status = Self::handle_one_check(
                        url,
                        self.hosts,
                        self.delayer,
                        &mut self.resolver_cache_ipv4,
                        self.http_client,
                        self.experimental_http_client,
                    )
                    .await;
                }

                histogram!("repology_linkchecker_checker_check_duration_seconds")
                    .record((Instant::now() - check_start).as_secs_f64());
                counter!("repology_linkchecker_checker_processed_total", "priority" => task.priority.as_str(), "class" => "Checked").increment(1);
            }
        } else {
            ipv4_status = if self.disable_ipv4 {
                LinkStatus::ProtocolDisabled.into()
            } else {
                LinkStatus::InvalidUrl.into()
            };
            ipv6_status = if self.disable_ipv6 {
                LinkStatus::ProtocolDisabled.into()
            } else {
                LinkStatus::InvalidUrl.into()
            };
            counter!("repology_linkchecker_checker_processed_total", "priority" => task.priority.as_str(), "class" => "InvalidUrl").increment(1);
        };

        let now = Utc::now();

        let check_result = CheckResult {
            id: task.id,
            check_time: now,
            next_check: now + host_settings.generate_recheck_time(recheck_case),
            ipv4: ipv4_status,
            ipv6: ipv6_status,
        };

        if let Some(last_checked) = &task.last_checked {
            histogram!("repology_linkchecker_checker_check_period_seconds")
                .record((now - last_checked).as_seconds_f64());
        }

        counter!(
            "repology_linkchecker_checker_statuses_total",
            "protocol" => "ipv4",
            "status" => check_result.ipv4.status.to_string(),
            "priority" => task.priority.as_str()
        )
        .increment(1);

        counter!(
            "repology_linkchecker_checker_statuses_total",
            "protocol" => "ipv6",
            "status" => check_result.ipv6.status.to_string(),
            "priority" => task.priority.as_str()
        )
        .increment(1);

        {
            // note that we only compare statuses here
            let old = LinkStatus::pick_from46(task.prev_ipv4_status, task.prev_ipv6_status);
            let new = LinkStatus::pick_from46(check_result.ipv4.status, check_result.ipv6.status);

            let is_breakage = old.is_success() == Some(true) && new.is_success() == Some(false);
            let is_new_broken = old.is_success() == None && new.is_success() == Some(false);
            let is_recovery = old.is_success() == Some(false) && new.is_success() == Some(true);

            if is_breakage || is_new_broken || is_recovery {
                let formatted_old = old.to_string();
                let formatted_new = new.to_string();

                if is_breakage {
                    counter!("repology_linkchecker_checker_status_changes_total", "kind" => "Link breakage").increment(1);
                    warn!(
                        url = task.url,
                        old = formatted_old,
                        new = formatted_new,
                        "link broke"
                    );
                }
                if is_new_broken {
                    counter!("repology_linkchecker_checker_status_changes_total", "kind" => "New broken link").increment(1);
                    warn!(
                        url = task.url,
                        old = formatted_old,
                        new = formatted_new,
                        "new broken link"
                    );
                }
                if is_recovery {
                    let recovery_duration =
                        task.last_success.map(|last_success| now - last_success);
                    counter!("repology_linkchecker_checker_status_changes_total", "kind" => "Link recovery").increment(1);
                    if let Some(duration) = recovery_duration {
                        histogram!("repology_linkchecker_checker_link_recovery_duration_seconds")
                            .record(duration.as_seconds_f64());
                    }
                    info!(
                        url = task.url,
                        old = formatted_old,
                        new = formatted_new,
                        recovery_duration = recovery_duration.map(chrono::Duration::as_seconds_f64),
                        failure_streak = task.failure_streak.unwrap_or_default(),
                        "link recovered"
                    );
                }
            }
        }

        check_result
    }
}
