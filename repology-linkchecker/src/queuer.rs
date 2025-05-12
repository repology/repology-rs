// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use metrics::{counter, gauge};
use tokio::task::{self};
use tracing::{debug, info};
use url::Url;

use crate::checker::CheckPriority;
use crate::checker::{CheckTask, Checker};
use crate::config::{
    DEFAULT_MAX_BUCKETS, DEFAULT_MAX_QUEUED_URLS, DEFAULT_MAX_QUEUED_URLS_PER_BUCKET,
};
use crate::delayer::Delayer;
use crate::hosts::Hosts;
use crate::http_client::HttpClient;
use crate::resolver::Resolver;
use crate::updater::Updater;

const OLD_BUCKET_AGE_THRESHOLD: Duration = Duration::from_secs(600);
const OLD_BUCKET_LOG_PERIOD: Duration = Duration::from_secs(300);

struct Bucket {
    tasks: VecDeque<CheckTask>,
    task_ids: HashSet<i32>,
    create_time: Instant,
    num_deferred: usize,
}

impl Default for Bucket {
    fn default() -> Self {
        Self {
            tasks: Default::default(),
            task_ids: Default::default(),
            create_time: Instant::now(),
            num_deferred: 0,
        }
    }
}

#[derive(Default)]
struct State {
    num_queued_tasks: usize,
    buckets: HashMap<String, Bucket>,
    max_bucket_age: Duration,
}

pub struct Queuer<R, ER> {
    state: Arc<Mutex<State>>,
    resolver: Arc<Resolver>,
    hosts: Arc<Hosts>,
    delayer: Arc<Delayer>,
    http_client: Arc<R>,
    experimental_http_client: Arc<ER>,
    updater: Arc<Updater>,
    max_queued_urls: usize,
    max_queued_urls_per_bucket: usize,
    max_buckets: usize,
    disable_ipv4: bool,
    disable_ipv6: bool,
    satisfy_with_ipv6: bool,
}

impl<R, ER> Queuer<R, ER>
where
    R: HttpClient + Send + Sync + 'static,
    ER: HttpClient + Send + Sync + 'static,
{
    pub fn new(
        resolver: Resolver,
        hosts: Hosts,
        delayer: Delayer,
        http_client: R,
        experimental_http_client: ER,
        updater: Updater,
    ) -> Self {
        Self {
            state: Default::default(),
            resolver: Arc::new(resolver),
            hosts: Arc::new(hosts),
            delayer: Arc::new(delayer),
            http_client: Arc::new(http_client),
            experimental_http_client: Arc::new(experimental_http_client),
            updater: Arc::new(updater),
            max_queued_urls: DEFAULT_MAX_QUEUED_URLS,
            max_queued_urls_per_bucket: DEFAULT_MAX_QUEUED_URLS_PER_BUCKET,
            max_buckets: DEFAULT_MAX_BUCKETS,
            disable_ipv4: false,
            disable_ipv6: false,
            satisfy_with_ipv6: false,
        }
    }

    pub fn with_max_queued_urls(mut self, max_queued_urls: usize) -> Self {
        self.max_queued_urls = max_queued_urls;
        self
    }

    pub fn with_max_queued_urls_per_bucket(mut self, max_queued_urls_per_bucket: usize) -> Self {
        self.max_queued_urls_per_bucket = max_queued_urls_per_bucket;
        self
    }

    pub fn with_max_buckets(mut self, max_buckets: usize) -> Self {
        self.max_buckets = max_buckets;
        self
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

    // many args is legal here IMO, however may be reducer a bit by
    // grouping parameters which are only passed through to Checker
    #[allow(clippy::too_many_arguments)]
    async fn handle_bucket(
        bucket_key: String,
        state: Weak<Mutex<State>>,
        resolver: Arc<Resolver>,
        hosts: Arc<Hosts>,
        delayer: Arc<Delayer>,
        http_client: Arc<R>,
        experimental_http_client: Arc<ER>,
        updater: Arc<Updater>,
        disable_ipv4: bool,
        disable_ipv6: bool,
        satisfy_with_ipv6: bool,
    ) {
        let mut num_processed: usize = 0;
        let mut last_log_time = Instant::now();

        let mut checker = Checker::new(
            &resolver,
            &hosts,
            &delayer,
            &*http_client,
            &*experimental_http_client,
        )
        .with_disable_ipv4(disable_ipv4)
        .with_disable_ipv6(disable_ipv6)
        .with_satisfy_with_ipv6(satisfy_with_ipv6);

        // Give a newborn bucket some time to fill up, otherwise buckets which
        // tend to process tasks without delays (e.g. when a host is skipped)
        // will process a few tasks and die right away just to be born again
        tokio::time::sleep(Duration::from_secs(1)).await;

        loop {
            let Some(state) = state.upgrade() else {
                break;
            };

            let task = {
                let mut state = state.lock().unwrap();

                let num_buckets = state.buckets.len();
                let bucket = state
                    .buckets
                    .get_mut(&bucket_key)
                    .expect("bucket is expected to exist as long as the task exists");

                let now = Instant::now();

                let Some(task) = bucket.tasks.pop_front() else {
                    debug!(
                        key = bucket_key,
                        num_processed,
                        num_deferred = bucket.num_deferred,
                        num_buckets_remaining = num_buckets - 1,
                        seconds_per_task =
                            (now - bucket.create_time).as_secs_f64() / num_processed as f64,
                        "bucket exhausted"
                    );
                    state.buckets.remove(&bucket_key);
                    state.max_bucket_age = state
                        .buckets
                        .values()
                        .map(|bucket| now - bucket.create_time)
                        .max()
                        .unwrap_or_default();
                    gauge!("repology_linkchecker_queuer_buckets_max_age_seconds")
                        .set(state.max_bucket_age.as_secs_f64());
                    gauge!("repology_linkchecker_queuer_buckets_total")
                        .set(state.buckets.len() as f64);
                    break;
                };

                let bucket_age = now - bucket.create_time;
                if bucket_age > OLD_BUCKET_AGE_THRESHOLD
                    && now - last_log_time > OLD_BUCKET_LOG_PERIOD
                {
                    info!(
                        key = bucket_key,
                        age = ?bucket_age,
                        num_queued = bucket.tasks.len(),
                        num_processed = num_processed,
                        num_deferred = bucket.num_deferred,
                        seconds_per_task =
                            bucket_age.as_secs_f64() / num_processed as f64,
                        "old bucket"
                    );
                    last_log_time = now;
                }
                state.max_bucket_age = state.max_bucket_age.max(bucket_age);
                gauge!("repology_linkchecker_queuer_buckets_max_age_seconds")
                    .set(state.max_bucket_age.as_secs_f64());

                state.num_queued_tasks -= 1;
                gauge!("repology_linkchecker_queuer_tasks_queued_total")
                    .set(state.num_queued_tasks as f64);
                task
            };

            let task_id = task.id;

            updater.push(checker.check(task).await).await;

            state
                .lock()
                .unwrap()
                .buckets
                .get_mut(&bucket_key)
                .expect("bucket we've just retrieved task from should still exist")
                .task_ids
                .remove(&task_id);

            num_processed += 1;
            counter!("repology_linkchecker_queuer_tasks_total", "state" => "processed")
                .increment(1);
        }
    }

    // silences false positive, see the code
    #[expect(clippy::await_holding_lock)]
    pub async fn try_put(&self, task: CheckTask) -> bool {
        let mut first_iteration = true;
        loop {
            if !first_iteration {
                // if we haven't succeeded on the second iteration, Queuer
                // is overflown and we need to wait until resources are freed
                info!("waiting for some slots to free up");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            first_iteration = false;

            let mut state = self.state.lock().unwrap();

            if state.num_queued_tasks >= self.max_queued_urls {
                // total queued URLs limit reached
                continue;
            }

            let maybe_parsed_url = Url::parse(&task.url).ok();
            let (bucket_key, host_settings) =
                if let Some(host) = maybe_parsed_url.as_ref().and_then(|url| url.host_str()) {
                    (
                        self.hosts.get_aggregation(host),
                        self.hosts.get_settings(host),
                    )
                } else {
                    // TODO: immediately update with InvalidUrl?
                    ("", self.hosts.get_default_settings())
                };

            let bucket = if let Some(bucket) = state.buckets.get_mut(bucket_key) {
                if bucket.task_ids.contains(&task.id) {
                    // already in the queue
                    counter!("repology_linkchecker_queuer_tasks_total", "state" => "already_queued").increment(1);
                    return false;
                }
                if bucket.tasks.len() >= self.max_queued_urls_per_bucket {
                    if task.last_checked.is_none() && task.priority == CheckPriority::Manual {
                        // don't defer unchecked manual links
                        counter!("repology_linkchecker_queuer_tasks_total", "state" => "retained", "bucket" => bucket_key.to_string()).increment(1);
                    } else {
                        // Some hosts are just too slow to check, and their queues are quickly
                        // overflown. We don't want to block here, instead we defer tasks
                        bucket.num_deferred += 1;
                        drop(state); // no longer needed, and don't hold a lock across await point
                        // clippy::await_holding_lock false positive about holding a lock across await
                        // (but we don't as the lock is dropped a line above), silenced at function level
                        // See https://github.com/rust-lang/rust-clippy/issues/9683
                        self.updater
                            .defer_by(task.id, host_settings.generate_defer_time(task.priority))
                            .await;
                        counter!("repology_linkchecker_queuer_tasks_total", "state" => "deferred", "bucket" => bucket_key.to_string()).increment(1);
                    }
                    return false;
                }
                bucket
            } else if state.buckets.len() < self.max_buckets {
                // Is there really no e.g. HashMap::get_or_insert_with() to avoid both
                // key consuming and double lookup?
                // Update: may use raw_entry_mut here
                gauge!("repology_linkchecker_queuer_buckets_total")
                    .set((state.buckets.len() + 1) as f64);
                let bucket = state.buckets.entry(bucket_key.to_string()).or_default();

                task::spawn(Self::handle_bucket(
                    bucket_key.to_string(),
                    Arc::downgrade(&self.state),
                    Arc::clone(&self.resolver),
                    Arc::clone(&self.hosts),
                    Arc::clone(&self.delayer),
                    Arc::clone(&self.http_client),
                    Arc::clone(&self.experimental_http_client),
                    Arc::clone(&self.updater),
                    self.disable_ipv4,
                    self.disable_ipv6,
                    self.satisfy_with_ipv6,
                ));

                debug!(key = bucket_key, "bucket created");

                bucket
            } else {
                // buckets limit reached
                continue;
            };

            let task_id = task.id;
            bucket.tasks.push_back(task);
            bucket.task_ids.insert(task_id);
            state.num_queued_tasks += 1;
            gauge!("repology_linkchecker_queuer_tasks_queued_total")
                .set(state.num_queued_tasks as f64);
            counter!("repology_linkchecker_queuer_tasks_total", "state" => "enqueued").increment(1);
            return true;
        }
    }
}
