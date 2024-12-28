// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};

pub fn parse_utc_datetime(date: &str) -> Result<DateTime<Utc>> {
    Ok(NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3f")?.and_utc())
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_utc_datetime() {
        assert_eq!(
            parse_utc_datetime("2024-01-02T03:04:05.006").unwrap(),
            DateTime::<Utc>::from_timestamp(1704164645, 6000000).unwrap()
        );
    }
}
