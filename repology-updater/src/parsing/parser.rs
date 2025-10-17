// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use crate::parsing::sink::PackageSink;

pub trait PackageParser {
    fn parse(&self, path: &Path, sink: &mut dyn PackageSink) -> anyhow::Result<()>;
}
