// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use repology_updater::parsing::parsers::create_parser;

use repology_updater::fetching::fetchers::create_fetcher_options_yaml;

use crate::config::RawCommands;

use std::time::Instant;

pub fn raw_command(command: &RawCommands) {
    match command {
        RawCommands::Parse {
            parser_name,
            state_path,
            print,
        } => {
            let mut num_packages: u64 = 0;
            let parser = create_parser(parser_name).unwrap();
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

            {
                let state_path = state_path.to_path_buf();
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(async move {
                        let handle = fetcher.fetch(&state_path).await.unwrap();
                        handle.accept().await.unwrap();
                    });
            }

            let duration = Instant::now() - start;
            eprintln!("Fetched in {:.2} sec", duration.as_secs_f64());
        }
        RawCommands::FetchParse {
            parser_name,
            fetcher_name,
            state_path,
            fetcher_options,
            print,
        } => {
            let mut num_packages: u64 = 0;
            let parser = create_parser(parser_name).unwrap();
            let fetcher = create_fetcher_options_yaml(fetcher_name, fetcher_options).unwrap();

            {
                let state_path = state_path.to_path_buf();
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(async move {
                        let handle = fetcher.fetch(&state_path).await.unwrap();

                        let mut res = Ok(());
                        rayon::scope(|scope| {
                            scope.spawn(|_| {
                                res = parser.parse(handle.path(), &mut |package_maker| {
                                    num_packages += 1;
                                    if *print {
                                        println!("{:#?}", package_maker.finalize()?)
                                    }
                                    Ok(())
                                });
                            });
                        });

                        res.unwrap();

                        handle.accept().await.unwrap();
                    });
            }
        }
    }
}
