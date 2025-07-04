// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use repology_updater::parsing::parser::PackageAccumulator;
use repology_updater::parsing::parser::PackageParser as _;
use repology_updater::parsing::parsers::freebsd::FreeBsdParser;

#[test]
fn test_freebsd() {
    let mut packages = PackageAccumulator::default();
    FreeBsdParser {}
        .parse(Path::new("tests/parser_tests/freebsd/INDEX"), &mut packages)
        .unwrap();
    insta::assert_debug_snapshot!(packages.packages);
}

#[test]
fn test_error_missing_field() {
    let mut packages = PackageAccumulator::default();
    insta::assert_debug_snapshot!(FreeBsdParser {}.parse(
        Path::new("tests/parser_tests/freebsd/INDEX-1"),
        &mut packages,
    ))
}

#[test]
fn test_error_extra_field() {
    let mut packages = PackageAccumulator::default();
    insta::assert_debug_snapshot!(FreeBsdParser {}.parse(
        Path::new("tests/parser_tests/freebsd/INDEX-2"),
        &mut packages,
    ))
}

#[test]
fn test_error_missing_version() {
    let mut packages = PackageAccumulator::default();
    insta::assert_debug_snapshot!(FreeBsdParser {}.parse(
        Path::new("tests/parser_tests/freebsd/INDEX-3"),
        &mut packages,
    ))
}

#[test]
fn test_error_missing_name() {
    let mut packages = PackageAccumulator::default();
    insta::assert_debug_snapshot!(FreeBsdParser {}.parse(
        Path::new("tests/parser_tests/freebsd/INDEX-4"),
        &mut packages,
    ))
}

#[test]
fn test_error_bad_package_format() {
    let mut packages = PackageAccumulator::default();
    insta::assert_debug_snapshot!(FreeBsdParser {}.parse(
        Path::new("tests/parser_tests/freebsd/INDEX-5"),
        &mut packages,
    ))
}
