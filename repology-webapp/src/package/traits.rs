// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use repology_common::{PackageFlags, PackageStatus};

pub trait PackageWithVersion {
    fn version(&self) -> &str;
}

pub trait PackageWithFlags {
    fn flags(&self) -> PackageFlags;
}

pub trait PackageWithStatus {
    fn status(&self) -> PackageStatus;
}

pub trait PackageWithRepositoryName {
    fn repository_name(&self) -> &str;
}
