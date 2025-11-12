// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::{Duration, Instant};

use crate::config::{CliArgs, RawCommands};
use crate::fetching::fetchers::create_fetcher_options_yaml;
use crate::fetching::http::Http;
use crate::parsing::parsers::create_parser_options_yaml;
use crate::repositories_config::RepositoriesConfig;
use crate::ruleset::Ruleset;

fn init_logging(debug: bool) {
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let layer = tracing_subscriber::fmt::Layer::new().with_timer(
        tracing_subscriber::fmt::time::ChronoLocal::new(String::from("%F %T%.6f")),
    );

    tracing_subscriber::registry()
        .with(EnvFilter::new(if debug { "debug" } else { "info" }))
        .with(layer)
        .init();
}

async fn raw_command_async(command: &RawCommands, args: &CliArgs) {
    init_logging(args.debug);

    match command {
        RawCommands::Parse {
            parser_name,
            parser_options,
            state_path,
            print,
        } => {
            let mut num_packages: u64 = 0;
            let parser = create_parser_options_yaml(
                parser_name,
                parser_options.as_deref().unwrap_or_default(),
            )
            .unwrap();
            let start = Instant::now();
            parser
                .parse(state_path, &mut |package_maker| {
                    num_packages += 1;
                    if *print {
                        println!("{:#?}", package_maker.finalize()?)
                    }
                    Ok(())
                })
                .unwrap();
            let duration = Instant::now() - start;
            eprintln!(
                "Parsed {} package(s) in {:.2} sec ({:.2} packages/sec)",
                num_packages,
                duration.as_secs_f64(),
                num_packages as f64 / duration.as_secs_f64()
            );
        }
        RawCommands::Fetch {
            fetcher_name,
            state_path,
            fetcher_options,
        } => {
            let fetcher = create_fetcher_options_yaml(fetcher_name, fetcher_options).unwrap();
            let start = Instant::now();

            let fetch_result = fetcher.fetch(state_path, &Http::default()).await.unwrap();
            fetch_result.accept().await.unwrap();

            let duration = Instant::now() - start;
            eprintln!("Fetched in {:.2} sec", duration.as_secs_f64());
        }
        RawCommands::FetchParse {
            parser_name,
            parser_options,
            fetcher_name,
            fetcher_options,
            state_path,
            print,
        } => {
            let parser = create_parser_options_yaml(
                parser_name,
                parser_options.as_deref().unwrap_or_default(),
            )
            .unwrap();
            let fetcher = create_fetcher_options_yaml(fetcher_name, fetcher_options).unwrap();

            let start = Instant::now();
            let fetch_result = fetcher.fetch(state_path, &Http::default()).await.unwrap();
            let duration = Instant::now() - start;

            eprintln!("Fetched in {:.2} sec", duration.as_secs_f64());

            let res: anyhow::Result<(u64, Duration)> = {
                let print = *print;
                let path = fetch_result.state_path.clone();
                async_rayon::spawn(move || {
                    let mut num_packages: u64 = 0;
                    let start = Instant::now();
                    parser.parse(&path, &mut |package_maker| {
                        num_packages += 1;
                        if print {
                            println!("{:#?}", package_maker.finalize()?)
                        }
                        Ok(())
                    })?;
                    let parse_duration = Instant::now() - start;
                    Ok((num_packages, parse_duration))
                })
                .await
            };

            let (num_packages, parse_duration) = match res {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Parsing failed ({:?}), rejecting freshly fetched data", e);
                    return;
                }
            };

            fetch_result.accept().await.unwrap();

            eprintln!(
                "Parsed {} package(s) in {:.2} sec ({:.2} packages/sec)",
                num_packages,
                parse_duration.as_secs_f64(),
                num_packages as f64 / parse_duration.as_secs_f64()
            );
        }
        RawCommands::DumpRuleset { ruleset_path } => {
            let ruleset = Ruleset::parse(ruleset_path).unwrap();
            for rule in &ruleset.rules {
                print!("- {}", rule.to_yaml().unwrap());
            }
        }
        RawCommands::DumpRepositories { repositories_path } => {
            print!(
                "{}",
                RepositoriesConfig::parse(repositories_path)
                    .unwrap()
                    .to_yaml()
                    .unwrap()
            );
        }
    }
}

pub fn raw_command(command: &RawCommands, args: &CliArgs) {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(raw_command_async(command, args));
}
