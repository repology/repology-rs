// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use repology_updater::parsing::parser::PackageAccumulator;
use repology_updater::parsing::parser::PackageParser as _;
use repology_updater::parsing::parsers::freebsd::FreeBsdParser;

#[test]
#[ignore]
fn test_freebsd() {
    let mut packages = PackageAccumulator::default();
    FreeBsdParser {}
        .parse(Path::new("tests/parser_tests/freebsd/ok"), &mut packages)
        .unwrap();
    insta::assert_debug_snapshot!(packages.packages);
}

#[test]
fn test_error_missing_field() {
    let mut packages = PackageAccumulator::default();
    insta::assert_debug_snapshot!(FreeBsdParser {}.parse(
        Path::new("tests/parser_tests/freebsd/error1"),
        &mut packages,
    ))
}

#[test]
fn test_error_extra_field() {
    let mut packages = PackageAccumulator::default();
    insta::assert_debug_snapshot!(FreeBsdParser {}.parse(
        Path::new("tests/parser_tests/freebsd/error2"),
        &mut packages,
    ))
}

#[test]
fn test_error_missing_version() {
    let mut packages = PackageAccumulator::default();
    insta::assert_debug_snapshot!(FreeBsdParser {}.parse(
        Path::new("tests/parser_tests/freebsd/error3"),
        &mut packages,
    ))
}

#[test]
fn test_error_missing_name() {
    let mut packages = PackageAccumulator::default();
    insta::assert_debug_snapshot!(FreeBsdParser {}.parse(
        Path::new("tests/parser_tests/freebsd/error4"),
        &mut packages,
    ))
}

#[test]
fn test_error_bad_package_format() {
    let mut packages = PackageAccumulator::default();
    insta::assert_debug_snapshot!(FreeBsdParser {}.parse(
        Path::new("tests/parser_tests/freebsd/error5"),
        &mut packages,
    ))
}

#[test]
fn test_error_bad_package_path() {
    let mut packages = PackageAccumulator::default();
    insta::assert_debug_snapshot!(FreeBsdParser {}.parse(
        Path::new("tests/parser_tests/freebsd/error6"),
        &mut packages,
    ))
}
