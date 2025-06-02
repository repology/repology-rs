// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;

use crate::checker::CheckPriority;

#[derive(Deserialize, Debug, Clone)]
pub struct HostSettings {
    pub delay: Duration,
    pub timeout: Duration,
    pub recheck_manual: Duration,
    pub recheck_generated: Duration,
    pub recheck_unsampled: Duration,
    pub recheck_splay: f32,
    pub skip: bool,
    pub aggregate: bool,
    pub blacklist: bool,
    pub disable_ipv6: bool,
    pub disable_head: bool,
    pub generated_sampling_percentage: u8,
    pub is: Option<String>,
}

impl Default for HostSettings {
    fn default() -> Self {
        Self {
            delay: Duration::from_secs(3),
            timeout: Duration::from_secs(20),
            recheck_manual: Duration::from_days(7),
            recheck_generated: Duration::from_days(14),
            recheck_unsampled: Duration::from_days(60),
            recheck_splay: 1.0,
            skip: false,
            aggregate: false,
            blacklist: false,
            disable_ipv6: false,
            disable_head: false,
            generated_sampling_percentage: 100,
            is: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RecheckCase {
    Manual,
    Generated,
    Unsampled,
}

impl From<CheckPriority> for RecheckCase {
    fn from(priority: CheckPriority) -> Self {
        match priority {
            CheckPriority::Manual => Self::Manual,
            CheckPriority::Generated => Self::Generated,
        }
    }
}

impl HostSettings {
    pub fn generate_recheck_interval(&self, case: RecheckCase) -> Duration {
        let recheck_period = match case {
            RecheckCase::Manual => self.recheck_manual,
            RecheckCase::Generated => self.recheck_generated,
            RecheckCase::Unsampled => self.recheck_unsampled,
        };
        // [recheck, recheck + splay)
        recheck_period.mul_f32(1.0 + self.recheck_splay * rand::random::<f32>())
    }

    pub fn generate_fast_failure_recheck_interval(
        &self,
        case: RecheckCase,
        failure_streak: u16,
    ) -> Option<Duration> {
        let interval = match case {
            RecheckCase::Manual => {
                const RECHECK_INTERVALS: &[Duration] = &[
                    Duration::from_hours(1),
                    Duration::from_hours(4),
                    Duration::from_days(1),
                    Duration::from_days(3),
                ];
                RECHECK_INTERVALS.get(failure_streak as usize)
            }
            RecheckCase::Generated => {
                const RECHECK_INTERVALS: &[Duration] =
                    &[Duration::from_hours(1), Duration::from_days(1)];
                RECHECK_INTERVALS.get(failure_streak as usize)
            }
            _ => None,
        };

        if let Some(interval) = interval {
            // [recheck, recheck + splay)
            Some(interval.mul_f32(1.0 + self.recheck_splay * rand::random::<f32>()))
        } else {
            None
        }
    }

    pub fn generate_defer_interval(&self, priority: CheckPriority) -> Duration {
        let recheck_period = match priority {
            CheckPriority::Manual => self.recheck_manual,
            CheckPriority::Generated => self.recheck_generated,
        };
        // [0, recheck + splay), because we don't want to produce gaps
        recheck_period.mul_f32((1.0 + self.recheck_splay) * rand::random::<f32>())
    }
}

pub struct Hosts {
    default_host_settings: HostSettings,
    host_settings: HashMap<String, HostSettings>,
}

impl Hosts {
    pub fn new(
        default_host_settings: HostSettings,
        host_settings: HashMap<String, HostSettings>,
    ) -> Self {
        Self {
            default_host_settings,
            host_settings,
        }
    }

    pub fn get_settings<'a>(&'a self, hostname: &str) -> &'a HostSettings {
        let mut current_hostname = hostname;
        loop {
            if let Some(host_settings) = self.host_settings.get(current_hostname) {
                if let Some(target) = &host_settings.is {
                    return self
                        .host_settings
                        .get(target)
                        .unwrap_or(&self.default_host_settings);
                }
                return host_settings;
            }
            if let Some(separator_pos) = current_hostname.find('.') {
                current_hostname = &current_hostname[separator_pos + 1..];
            } else {
                return &self.default_host_settings;
            };
        }
    }

    pub fn get_default_settings(&self) -> &HostSettings {
        &self.default_host_settings
    }

    pub fn get_aggregation<'a>(&'a self, hostname: &'a str) -> &'a str {
        let hostname = hostname.strip_prefix("www.").unwrap_or(hostname);
        let mut current_hostname = hostname;
        loop {
            if let Some(host_settings) = self.host_settings.get(current_hostname) {
                if let Some(target) = &host_settings.is {
                    return target;
                }
                if host_settings.aggregate {
                    return current_hostname;
                }
            }
            if let Some(separator_pos) = current_hostname.find('.') {
                current_hostname = &current_hostname[separator_pos + 1..];
            } else {
                return hostname;
            };
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_base() {
        let mut hosts: HashMap<String, HostSettings> = Default::default();
        hosts.insert(
            "foo.example.com".to_string(),
            HostSettings {
                generated_sampling_percentage: 42,
                ..Default::default()
            },
        );
        let hosts = Hosts::new(HostSettings::default(), hosts);

        assert_eq!(hosts.get_aggregation("example.com"), "example.com");
        assert_eq!(hosts.get_aggregation("foo.example.com"), "foo.example.com");
        assert_eq!(hosts.get_aggregation("www.example.com"), "example.com");
        assert_eq!(
            hosts.get_aggregation("www.foo.example.com"),
            "foo.example.com"
        );
        assert_eq!(
            hosts.get_aggregation("bar.foo.example.com"),
            "bar.foo.example.com"
        );

        assert_eq!(
            hosts
                .get_settings("example.com")
                .generated_sampling_percentage,
            100
        );
        assert_eq!(
            hosts
                .get_settings("foo.example.com")
                .generated_sampling_percentage,
            42
        );
        assert_eq!(
            hosts
                .get_settings("www.example.com")
                .generated_sampling_percentage,
            100
        );
        assert_eq!(
            hosts
                .get_settings("www.foo.example.com")
                .generated_sampling_percentage,
            42
        );
        assert_eq!(
            hosts
                .get_settings("bar.foo.example.com")
                .generated_sampling_percentage,
            42
        );
    }

    #[test]
    fn test_aggregate() {
        let mut hosts: HashMap<String, HostSettings> = Default::default();
        hosts.insert(
            "github.io".to_string(),
            HostSettings {
                aggregate: true,
                generated_sampling_percentage: 42,
                ..Default::default()
            },
        );
        let hosts = Hosts::new(HostSettings::default(), hosts);

        assert_eq!(hosts.get_aggregation("github.io"), "github.io");
        assert_eq!(hosts.get_aggregation("www.github.io"), "github.io");
        assert_eq!(hosts.get_aggregation("foo.github.io"), "github.io");
        assert_eq!(hosts.get_aggregation("www.foo.github.io"), "github.io");
        assert_eq!(hosts.get_aggregation("bar.foo.github.io"), "github.io");

        assert_eq!(
            hosts
                .get_settings("github.io")
                .generated_sampling_percentage,
            42
        );
        assert_eq!(
            hosts
                .get_settings("www.github.io")
                .generated_sampling_percentage,
            42
        );
        assert_eq!(
            hosts
                .get_settings("foo.github.io")
                .generated_sampling_percentage,
            42
        );
        assert_eq!(
            hosts
                .get_settings("www.foo.github.io")
                .generated_sampling_percentage,
            42
        );
        assert_eq!(
            hosts
                .get_settings("bar.foo.github.io")
                .generated_sampling_percentage,
            42
        );
    }

    #[test]
    fn test_is() {
        let mut hosts: HashMap<String, HostSettings> = Default::default();
        hosts.insert(
            "github.io".to_string(),
            HostSettings {
                is: Some("github.com".to_string()),
                ..Default::default()
            },
        );
        hosts.insert(
            "github.com".to_string(),
            HostSettings {
                generated_sampling_percentage: 42,
                ..Default::default()
            },
        );

        let hosts = Hosts::new(HostSettings::default(), hosts);

        assert_eq!(hosts.get_aggregation("github.io"), "github.com");
        assert_eq!(hosts.get_aggregation("www.github.io"), "github.com");
        assert_eq!(hosts.get_aggregation("foo.github.io"), "github.com");
        assert_eq!(hosts.get_aggregation("www.foo.github.io"), "github.com");
        assert_eq!(hosts.get_aggregation("bar.foo.github.io"), "github.com");

        assert_eq!(hosts.get_aggregation("github.com"), "github.com");
        assert_eq!(hosts.get_aggregation("www.github.com"), "github.com");
        assert_eq!(hosts.get_aggregation("foo.github.com"), "foo.github.com"); // no aggregation unless aggregate is also specified on github.com

        assert_eq!(
            hosts
                .get_settings("github.io")
                .generated_sampling_percentage,
            42
        );
        assert_eq!(
            hosts
                .get_settings("www.github.io")
                .generated_sampling_percentage,
            42
        );
        assert_eq!(
            hosts
                .get_settings("foo.github.io")
                .generated_sampling_percentage,
            42
        );
        assert_eq!(
            hosts
                .get_settings("www.foo.github.io")
                .generated_sampling_percentage,
            42
        );
        assert_eq!(
            hosts
                .get_settings("bar.foo.github.io")
                .generated_sampling_percentage,
            42
        );

        assert_eq!(
            hosts
                .get_settings("github.com")
                .generated_sampling_percentage,
            42
        );
        assert_eq!(
            hosts
                .get_settings("www.github.com")
                .generated_sampling_percentage,
            42
        );
        assert_eq!(
            hosts
                .get_settings("foo.github.com")
                .generated_sampling_percentage,
            42
        );
    }
}
