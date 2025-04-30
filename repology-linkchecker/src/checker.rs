// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Instant;

use chrono::Utc;
use http::status::StatusCode;
use metrics::{counter, histogram};
use tracing::{error, info, warn};
use url::Url;

use crate::delayer::Delayer;
use crate::hosts::{Hosts, RecheckCase};
use crate::http_client::{HttpClient, HttpMethod, HttpRequest, HttpResponse};
use crate::resolver::{IpVersion, Resolver, ResolverCache};
use crate::status::{HttpStatus, HttpStatusWithRedirect};
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
    pub prev_ipv4_status: Option<HttpStatus>,
    pub prev_ipv6_status: Option<HttpStatus>,
}

pub struct Checker<'a, R> {
    resolver_cache_ipv4: ResolverCache,
    resolver_cache_ipv6: ResolverCache,
    hosts: &'a Hosts,
    delayer: &'a Delayer,
    requester: &'a R,
    disable_ipv4: bool,
    disable_ipv6: bool,
    satisfy_with_ipv6: bool,
}

impl<'a, R> Checker<'a, R>
where
    R: HttpClient + Send + Sync + 'static,
{
    pub fn new(
        resolver: &Resolver,
        hosts: &'a Hosts,
        delayer: &'a Delayer,
        requester: &'a R,
    ) -> Self {
        Self {
            resolver_cache_ipv4: resolver.create_cache(IpVersion::Ipv4),
            resolver_cache_ipv6: resolver.create_cache(IpVersion::Ipv6),
            hosts,
            delayer,
            requester,
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

    async fn handle_one_request(
        url: &Url,
        hosts: &Hosts,
        delayer: &Delayer,
        resolver_cache: &mut ResolverCache,
        requester: &R,
    ) -> Result<HttpResponse, HttpStatus> {
        let host = url
            .host_str()
            .expect("only urls with host should end up here");

        counter!("repology_linkchecker_checker_http_requests_total").increment(1);

        match resolver_cache.lookup(host).await {
            Ok(address) => {
                let host_settings = hosts.get_settings(host);
                delayer
                    .reserve(hosts.get_aggregation(host), host_settings.delay)
                    .await;

                if !host_settings.disable_head {
                    let head_response = requester
                        .request(HttpRequest {
                            url: url.as_str().to_string(),
                            method: HttpMethod::Head,
                            address,
                            timeout: host_settings.timeout,
                        })
                        .await;

                    if head_response.status
                        != HttpStatus::Http(StatusCode::METHOD_NOT_ALLOWED.as_u16())
                    {
                        return Ok(head_response);
                    }
                }

                Ok(requester
                    .request(HttpRequest {
                        url: url.as_str().to_string(),
                        method: HttpMethod::Get,
                        address,
                        timeout: host_settings.timeout,
                    })
                    .await)
            }
            Err(resolve_error) => Err(HttpStatus::from_resolve_error(&resolve_error)),
        }
    }

    async fn handle_one_check(
        url: Url,
        hosts: &Hosts,
        delayer: &Delayer,
        resolver_cache: &mut ResolverCache,
        requester: &R,
    ) -> HttpStatusWithRedirect {
        let mut url = url;
        let mut num_redirects = 0;
        let mut had_temporary_redirect = false;
        let mut permanent_redirect_target: Option<String> = None;

        loop {
            let response =
                match Self::handle_one_request(&url, hosts, delayer, resolver_cache, requester)
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
                    return HttpStatus::InvalidUrl.into();
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
                return HttpStatusWithRedirect {
                    status,
                    // only save redirects for successes
                    redirect: permanent_redirect_target.filter(|_| status.is_success()),
                };
            }
            num_redirects += 1;
            if num_redirects >= MAX_REDIRECTS {
                return HttpStatus::TooManyRedirects.into();
            }
        }
    }

    pub async fn check(&mut self, task: CheckTask) -> CheckResult {
        let check_start = Instant::now();
        let mut check_result = CheckResult::default();
        let mut host_settings = self.hosts.get_default_settings();
        let mut recheck_case: RecheckCase = task.priority.into();

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
                counter!("repology_linkchecker_checker_processed_total", "priority" => task.priority.as_str(), "class" => "InvalidScheme").increment(1);
            } else if host_settings.skip {
                counter!("repology_linkchecker_checker_processed_total", "priority" => task.priority.as_str(), "class" => "Skipped").increment(1);
            } else if task.priority == CheckPriority::Generated
                && task.id % 100 >= host_settings.generated_sampling_percentage as i32
            {
                recheck_case = RecheckCase::Unsampled;
                counter!("repology_linkchecker_checker_processed_total", "priority" => task.priority.as_str(), "class" => "Unsampled").increment(1);
            } else if host_settings.blacklist {
                check_result.ipv4 =
                    Some(HttpStatus::Blacklisted.into()).filter(|_| !self.disable_ipv4);
                check_result.ipv6 =
                    Some(HttpStatus::Blacklisted.into()).filter(|_| !self.disable_ipv6);
            } else {
                let mut skip_ipv4 = false;

                if !self.disable_ipv6 && !host_settings.disable_ipv6 {
                    let status = Self::handle_one_check(
                        url.clone(),
                        self.hosts,
                        self.delayer,
                        &mut self.resolver_cache_ipv6,
                        self.requester,
                    )
                    .await;
                    skip_ipv4 = self.satisfy_with_ipv6 && status.status.is_success();
                    check_result.ipv6 = Some(status);
                }

                if !self.disable_ipv4 && !skip_ipv4 {
                    check_result.ipv4 = Some(
                        Self::handle_one_check(
                            url,
                            self.hosts,
                            self.delayer,
                            &mut self.resolver_cache_ipv4,
                            self.requester,
                        )
                        .await,
                    );
                }

                histogram!("repology_linkchecker_checker_check_duration_seconds")
                    .record((Instant::now() - check_start).as_secs_f64());
                counter!("repology_linkchecker_checker_processed_total", "priority" => task.priority.as_str(), "class" => "Checked").increment(1);
            }
        } else {
            check_result.ipv4 = Some(HttpStatus::InvalidUrl.into()).filter(|_| !self.disable_ipv4);
            check_result.ipv6 = Some(HttpStatus::InvalidUrl.into()).filter(|_| !self.disable_ipv6);
            counter!("repology_linkchecker_checker_processed_total", "priority" => task.priority.as_str(), "class" => "InvalidUrl").increment(1);
        };

        check_result.id = task.id;
        check_result.check_time = Utc::now();
        check_result.next_check =
            check_result.check_time + host_settings.generate_recheck_time(recheck_case);

        if let Some(status) = &check_result.ipv4 {
            counter!(
                "repology_linkchecker_checker_statuses_total",
                "protocol" => "ipv4",
                "status" => status.status.to_string(),
                "priority" => task.priority.as_str()
            )
            .increment(1);
        }

        if let Some(status) = &check_result.ipv6 {
            counter!(
                "repology_linkchecker_checker_statuses_total",
                "protocol" => "ipv6",
                "status" => status.status.to_string(),
                "priority" => task.priority.as_str()
            )
            .increment(1);
        }

        {
            // note that we only compare statuses here
            let old = HttpStatus::pick_from46(task.prev_ipv4_status, task.prev_ipv6_status);
            let new = HttpStatus::pick_from46(
                check_result.ipv4.as_ref().map(|status| status.status),
                check_result.ipv6.as_ref().map(|status| status.status),
            );

            let is_breakage = old.is_some_and(|status| status.is_success())
                && new.is_some_and(|status| !status.is_success());
            let is_new_broken = old.is_none() && new.is_some_and(|status| !status.is_success());
            let is_recovery = old.is_some_and(|status| !status.is_success())
                && new.is_some_and(|status| status.is_success());

            if is_breakage || is_new_broken || is_recovery {
                let formatted_old = old.map(|s| s.to_string()).unwrap_or_else(|| "-".into());
                let formatted_new = new.map(|s| s.to_string()).unwrap_or_else(|| "-".into());

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
                    counter!("repology_linkchecker_checker_status_changes_total", "kind" => "Link recovery").increment(1);
                    info!(
                        url = task.url,
                        old = formatted_old,
                        new = formatted_new,
                        "link recovered"
                    );
                }
            }
        }

        check_result
    }
}
