// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use repology_updater::ruleset::Ruleset;

#[test]
fn test_hier() {
    let path = Path::new("tests/ruleset_test_data/hier");
    let res = Ruleset::parse(path).expect("ruleset should be parsable");
    insta::assert_debug_snapshot!(res);
}

#[test]
fn test_simple_cases() {
    let path = Path::new("tests/ruleset_test_data/simple_cases");
    let res = Ruleset::parse(path).expect("ruleset should be parsable");
    insta::assert_debug_snapshot!(res);
}

#[test]
fn test_allflags() {
    let path = Path::new("tests/ruleset_test_data/allflags");
    let res = Ruleset::parse(path).expect("ruleset should be parsable");
    insta::assert_debug_snapshot!(res);
}

#[test]
fn test_error_duplicate_key() {
    let path = Path::new("tests/ruleset_test_data/error_duplicate_key");
    assert!(Ruleset::parse(path).is_err());
}
