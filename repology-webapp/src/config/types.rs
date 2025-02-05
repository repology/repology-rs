// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::net::IpAddr;
use std::str::FromStr;

use anyhow::{Result, anyhow};
use ip_network::IpNetwork;
use serde::Deserialize;

/// IP network type with custom parsing
///
/// This is a newtype for ip_network::IpNetwork, which allows parsing
/// from both network notation like `127.0.0.0/24` and plain IP addresses
/// like `127.0.0.1`.
pub struct MyIpNetwork(pub IpNetwork);

impl FromStr for MyIpNetwork {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.contains('/') {
            Self(s.parse()?)
        } else {
            Self(s.parse::<IpAddr>()?.into())
        })
    }
}

impl<'de> Deserialize<'de> for MyIpNetwork {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone)]
pub struct StaffAfkPeriod {
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
}

impl StaffAfkPeriod {
    pub fn is_active(&self) -> bool {
        let now = chrono::Utc::now().date_naive();
        self.from <= now && now <= self.to
    }
}

impl FromStr for StaffAfkPeriod {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((from, to)) = s.split_once(' ') {
            Ok(Self {
                from: from.parse()?,
                to: to.parse()?,
            })
        } else {
            Err(anyhow!("invalid date range format"))
        }
    }
}

impl<'de> Deserialize<'de> for StaffAfkPeriod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}
