// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

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
}

impl Args {
    pub fn should_update_all(&self) -> bool {
        !(self.should_update_cve || self.should_update_cpe)
    }
}
