// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

fn parse_duration(arg: &str) -> Result<std::time::Duration, std::num::ParseIntError> {
    Ok(match arg.split_at_checked(arg.len() - 1) {
        Some((seconds, "s")) => Duration::from_secs(seconds.parse()?),
        Some((minutes, "m")) => Duration::from_mins(minutes.parse()?),
        Some((hours, "h")) => Duration::from_hours(hours.parse()?),
        Some((days, "d")) => Duration::from_days(days.parse()?),
        Some((weeks, "w")) => Duration::from_weeks(weeks.parse()?),
        _ => Duration::from_secs(arg.parse()?),
    })
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// PostgreSQL database DSN
    #[arg(
        short = 'd',
        long,
        value_name = "DSN",
        default_value = "postgresql://repology@localhost/repology"
    )]
    pub dsn: String,

    /// Enable CVE update
    #[arg(long = "update-cve")]
    pub should_update_cve: bool,

    /// Enable CPE update
    #[arg(long = "update-cpe")]
    pub should_update_cpe: bool,

    /// Update period
    ///
    /// Specified as seconds or minutes/hours/days/weeks with m/h/d/w letter suffix
    /// correspondingly. Note that NVD requires this to be no less than two hours:
    ///
    /// https://nvd.nist.gov/developers/start-here#divBestPractices
    #[arg(short = 'u', long, value_name = "PERIOD", value_parser = parse_duration, default_value = "2h")]
    pub update_period: Duration,

    /// Run one update cycle, then exit (otherwise update continously)
    ///
    /// In this mode update period is ignored (so update is always performed),
    /// and application exits with failure code if any of the sources fails to
    /// update.
    #[arg(short = '1', long)]
    pub once_only: bool,

    /// Don't update repology tables after updating NVD data
    #[arg(long)]
    pub no_update_repology: bool,

    /// Path to log directory
    ///
    /// When specified, output is redirected to a log file in the
    /// given directory with daily rotation and 14 kept rotated files.
    #[arg(short = 'l', long, value_name = "PATH")]
    pub log_directory: Option<PathBuf>,

    /// Host/port for Prometheus metrics export endpoint
    #[arg(long, value_name = "HOST:PORT")]
    pub prometheus_export: Option<SocketAddr>,
}

impl Args {
    pub fn should_update_all(&self) -> bool {
        !(self.should_update_cve || self.should_update_cpe)
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("123"), Ok(Duration::from_secs(123)));
        assert_eq!(parse_duration("123s"), Ok(Duration::from_secs(123)));
        assert_eq!(parse_duration("1m"), Ok(Duration::from_secs(60)));
        assert_eq!(parse_duration("1h"), Ok(Duration::from_secs(3600)));
        assert_eq!(parse_duration("1d"), Ok(Duration::from_secs(86400)));
        assert_eq!(parse_duration("1w"), Ok(Duration::from_secs(604800)));
        assert!(parse_duration("1x").is_err());
    }
}
