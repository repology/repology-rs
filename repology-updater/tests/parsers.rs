// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use paste::paste;

use repology_updater::parsing::parser::PackageAccumulator;
use repology_updater::parsing::parser::PackageParser as _;
use repology_updater::parsing::parsers::*;

macro_rules! parser_test {
    ($parser_name:ident, $test_name:ident) => {
        paste! {
            #[test]
            fn [<test_ $parser_name:lower _ $test_name>]() {
                let mut packages = PackageAccumulator::default();
                let res = [<$parser_name Parser>] {}
                    .parse(Path::new(concat!("tests/parsers_test_data/", stringify!([<$parser_name:lower>]), "/", stringify!($test_name))), &mut packages).map(|_| packages.packages);
                insta::assert_debug_snapshot!(res);
            }
        }
    };
}

parser_test!(FreeBsd, ok);
parser_test!(FreeBsd, error_missing_field);
parser_test!(FreeBsd, error_extra_field);
parser_test!(FreeBsd, error_missing_version);
parser_test!(FreeBsd, error_missing_name);
parser_test!(FreeBsd, error_bad_package_format);
parser_test!(FreeBsd, error_bad_package_path);

parser_test!(TinCan, ok);

parser_test!(Yacp, ok);

parser_test!(StalIx, ok);
