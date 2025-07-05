// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use repology_updater::parsing::parser::PackageAccumulator;
use repology_updater::parsing::parser::PackageParser as _;
use repology_updater::parsing::parsers::yacp::YacpParser;

#[test]
fn test_yacp() {
    let mut packages = PackageAccumulator::default();
    YacpParser {}
        .parse(Path::new("tests/parser_tests/yacp/ok"), &mut packages)
        .unwrap();
    insta::assert_debug_snapshot!(packages.packages);
}
