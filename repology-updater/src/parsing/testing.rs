// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

macro_rules! parser_test {
    ($parser:expr, $parser_name:ident, $test_name:ident) => {
        paste::paste! {
            #[test]
            fn [<test_ $test_name>]() {
                eprintln!("{:?}", std::env::current_dir());
                println!("{:?}", std::env::current_dir());
                let path = std::path::Path::new(
                    concat!(
                        "src/parsing/parsers/fixtures/",
                        stringify!($parser_name),
                        "/",
                        stringify!($test_name)
                    )
                );
                let parser = $parser;
                let mut packages = vec![];
                let res = parser.parse(
                    path,
                    &mut |package_maker: $crate::parsing::package_maker::PackageMaker| {
                        packages.push(package_maker.finalize()?);
                        Ok(())
                    }
                ).map(|_| packages);
                if stringify!($test_name).starts_with("ok") {
                    insta::assert_debug_snapshot!(res.unwrap());
                } else if stringify!($test_name).starts_with("err") {
                    insta::assert_debug_snapshot!(res.unwrap_err());
                } else {
                    unreachable!();
                }
            }
        }
    };
}
