// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use repology_updater::parsing::parser::PackageAccumulator;
use repology_updater::parsing::parser::PackageParser as _;
use repology_updater::parsing::parsers::tincan::TinCanParser;

#[test]
#[ignore]
fn test_tincan() {
    let mut packages = PackageAccumulator::default();
    TinCanParser {}
        .parse(Path::new("tests/parser_tests/tincan/ok"), &mut packages)
        .unwrap();
    insta::assert_debug_snapshot!(packages.packages);
}
