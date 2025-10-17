// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use repology_updater::parsing::parsers::create_parser;
use repology_updater::parsing::sink::PackageSink;
use repology_updater::parsing::sinks::{PackageCounter, PackageDumper, PackageNullSink};

use crate::config::RawCommands;

use std::time::Instant;

pub fn raw_command(command: &RawCommands) {
    match command {
        RawCommands::Parse {
            parser_name,
            state_path,
            print,
        } => {
            let mut sink: Box<dyn PackageSink> = if *print {
                Box::new(PackageDumper::default())
            } else {
                Box::new(PackageNullSink::default())
            };
            let mut counter = PackageCounter::new(sink.as_mut());
            let parser = create_parser(parser_name).unwrap();
            let start = Instant::now();
            parser.parse(state_path, &mut counter).unwrap();
            let duration = Instant::now() - start;
            eprintln!(
                "Parsed {} package(s) in {:.2} sec ({:.2} packages/sec)",
                counter.count,
                duration.as_secs_f64(),
                counter.count as f64 / duration.as_secs_f64()
            );
        }
    }
}
