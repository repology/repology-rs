// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use hickory_resolver::config::ResolverConfig;
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::{ResolveError, TokioResolver};
use metrics::counter;

/// Run cache cleanup only if it exceeds the given size
const CACHE_CLEANUP_MIN_SIZE: usize = 32;

/// Perform cache cleanup every nth request
const CACHE_CLEANUP_PERIOD: usize = 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IpVersion {
    Ipv4,
    Ipv6,
}

pub struct Resolver {
    resolver: Arc<TokioResolver>,
}

impl Resolver {
    pub fn new() -> Self {
        let (sysconfig, options) = hickory_resolver::system_conf::read_system_conf().unwrap();

        let mut config = ResolverConfig::new();
        sysconfig
            .name_servers()
            .iter()
            .for_each(|name_server| config.add_name_server(name_server.clone()));

        let mut builder =
            TokioResolver::builder_with_config(config, TokioConnectionProvider::default());
        *builder.options_mut() = options; // do we really need system opts, or resolver's default, or custom?

        Self {
            resolver: Arc::new(builder.build()),
        }
    }

    pub fn create_cache(&self, ip_version: IpVersion) -> ResolverCache {
        ResolverCache {
            resolver: Arc::clone(&self.resolver),
            ip_version,
            cache: Default::default(),
            num_requests: 0,
        }
    }
}

struct CacheEntry {
    addresses: Vec<IpAddr>,
    valid_until: Instant,
    rr_index: usize,
}

pub struct ResolverCache {
    resolver: Arc<TokioResolver>,
    ip_version: IpVersion,
    cache: HashMap<String, CacheEntry>,
    num_requests: usize,
}

impl ResolverCache {
    pub async fn lookup(&mut self, domain: &str) -> std::result::Result<IpAddr, ResolveError> {
        let now = Instant::now();
        self.num_requests += 1;

        if self.cache.len() >= CACHE_CLEANUP_MIN_SIZE
            && self.num_requests % CACHE_CLEANUP_PERIOD == 0
        {
            self.cache.retain(|_, entry| entry.valid_until >= now);
        }

        if let Some(entry) = self.cache.get_mut(domain) {
            if entry.valid_until >= now {
                entry.rr_index = (entry.rr_index + 1) % entry.addresses.len();
                counter!("repology_linkchecker_resolver_requests_total", "type" => "Cached")
                    .increment(1);
                return Ok(entry.addresses[entry.rr_index]);
            } else {
                self.cache.remove(domain);
            }
        }

        // These arms look the same, but in fact work with different types
        // lookup: hickory_resolver::lookup::{Ipv4Lookup, Ipv6Lookup}
        // record: hickory_proto::rr::rdata::{a::A, aaaa::AAAA}
        let result: Result<(Vec<IpAddr>, Instant), ResolveError> = match self.ip_version {
            IpVersion::Ipv4 => self.resolver.ipv4_lookup(domain).await.map(|lookup| {
                (
                    lookup.iter().map(|record| record.0.into()).collect(),
                    lookup.valid_until(),
                )
            }),
            IpVersion::Ipv6 => self.resolver.ipv6_lookup(domain).await.map(|lookup| {
                (
                    lookup.iter().map(|record| record.0.into()).collect(),
                    lookup.valid_until(),
                )
            }),
        };

        match result {
            Ok((addresses, valid_until)) => {
                assert!(
                    !addresses.is_empty(),
                    "lookup is expected to contain at least one address"
                );
                let addr = addresses[0];
                self.cache.insert(
                    domain.to_string(),
                    CacheEntry {
                        addresses,
                        valid_until,
                        rr_index: 0,
                    },
                );
                counter!("repology_linkchecker_resolver_requests_total", "type" => "Lookup")
                    .increment(1);
                Ok(addr)
            }
            Err(err) => {
                counter!("repology_linkchecker_resolver_requests_total", "type" => "Failure")
                    .increment(1);
                Err(err)
            }
        }
    }
}
