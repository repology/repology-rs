// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use chrono::Datelike;
use sqlx::PgPool;

use crate::args::Args;
use crate::processors::cpe_dict::CpeDictProcessor;
use crate::processors::cve_feed::CveFeedProcessor;
use crate::processors::DatasourceProcessor;

pub struct Datasource {
    pub url: String,
    pub update_period: Duration,
    pub processor: Box<dyn DatasourceProcessor>,
}

pub fn generate_datasources(args: &Args, pool: PgPool) -> Vec<Datasource> {
    let update_all =
        !(args.update_fast_feed || args.update_slow_feeds || args.update_cpe_dictionary);

    let mut datasources: Vec<Datasource> = Default::default();

    const FAST_UPDATE_PERIOD: Duration = Duration::from_mins(10);
    const SLOW_UPDATE_PERIOD: Duration = Duration::from_days(1);

    if update_all || args.update_fast_feed {
        datasources.push(Datasource {
            url: "https://nvd.nist.gov/feeds/json/cve/1.1/nvdcve-1.1-modified.json.gz".into(),
            update_period: FAST_UPDATE_PERIOD,
            processor: Box::new(CveFeedProcessor::new(pool.clone())),
        });
    }

    if update_all || args.update_slow_feeds {
        // we don't care of timezones here, as normally fast feed contains
        // all the recent updates, and it doesn't matter if we enable
        let current_year = chrono::Utc::now().date_naive().year();

        for year in 2002..=current_year {
            datasources.push(Datasource {
                url: format!("https://nvd.nist.gov/feeds/json/cve/1.1/nvdcve-1.1-{year}.json.gz"),
                update_period: SLOW_UPDATE_PERIOD,
                processor: Box::new(CveFeedProcessor::new(pool.clone())),
            });
        }
    }

    if update_all || args.update_cpe_dictionary {
        datasources.push(Datasource {
            url:
                "https://nvd.nist.gov/feeds/xml/cpe/dictionary/official-cpe-dictionary_v2.3.xml.gz"
                    .into(),
            update_period: SLOW_UPDATE_PERIOD,
            processor: Box::new(CpeDictProcessor {}),
        });
    }

    datasources
}
