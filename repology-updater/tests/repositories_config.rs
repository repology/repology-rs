// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use repology_updater::repositories_config::RepositoriesConfig;

#[test]
fn test_allflags() {
    let path = Path::new("tests/repositories_config_test_data/allflags");
    let res = RepositoriesConfig::parse(path).expect("repositories config should be parsable");
    insta::assert_debug_snapshot!(res);
}
