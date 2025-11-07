// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use tokio::sync::{Mutex as TokioMutex, OwnedMutexGuard};
use url::Url;

type HostLock = Arc<TokioMutex<Option<Instant>>>;

#[derive(Clone, Default)]
pub struct FetchPoliteness {
    delay: Duration,
    hosts: Arc<StdMutex<HashMap<String, HostLock>>>,
}

impl FetchPoliteness {
    #[cfg_attr(not(test), expect(unused))] // will be configurable
    pub fn new(delay: Duration) -> Self {
        Self {
            delay,
            hosts: Default::default(),
        }
    }

    pub async fn acquire(&self, url: &str) -> HostPermit {
        // Note: invalid urls, if any, are just put in a single bucket
        // TODO: would be nice to strip subdomains, maybe with tld crate
        let key = Url::parse(url)
            .ok()
            .and_then(|url| url.host_str().map(|host| host.to_owned()))
            .unwrap_or_default();

        let lock = Arc::clone(self.hosts.lock().unwrap().entry(key).or_default());
        let guard = lock.lock_owned().await;
        let now = Instant::now();

        if let Some(last_freed) = *guard
            && now < last_freed + self.delay
        {
            tokio::time::sleep_until((last_freed + self.delay).into()).await;
        }

        HostPermit { guard }
    }
}

pub struct HostPermit {
    guard: OwnedMutexGuard<Option<Instant>>,
}

impl Drop for HostPermit {
    fn drop(&mut self) {
        *self.guard = Some(Instant::now());
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    async fn fetch_url(url: &str, politeness: FetchPoliteness) -> Duration {
        let start = Instant::now();
        let _guard = politeness.acquire(url).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        Instant::now() - start
    }

    #[tokio::test]
    async fn test_politeness() {
        let politeness = FetchPoliteness::new(Duration::from_millis(100));

        let (foo0, foo1, bar) = tokio::try_join!(
            tokio::spawn(fetch_url("https://foo.example.com/a", politeness.clone())),
            tokio::spawn(fetch_url("https://foo.example.com/b", politeness.clone())),
            tokio::spawn(fetch_url("https://bar.example.com/c", politeness.clone())),
        )
        .unwrap();

        let foo_longest = foo0.max(foo1);
        // 300+Δ ms = 100 ms fetch a + 100 ms delay + 100 ms fetch b
        assert!(foo_longest >= Duration::from_millis(300));
        assert!(foo_longest < Duration::from_millis(400));
        // 100+Δ ms = just a single fetch
        assert!(bar >= Duration::from_millis(100));
        assert!(bar < Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_bad_url() {
        let politeness = FetchPoliteness::new(Duration::from_millis(100));

        let (a, b) = tokio::try_join!(
            tokio::spawn(fetch_url("", politeness.clone())),
            tokio::spawn(fetch_url("Hello, world!", politeness.clone())),
        )
        .unwrap();

        assert!(a.max(b) >= Duration::from_millis(300));
        assert!(a.max(b) < Duration::from_millis(400));
    }
}
