// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, strum::Display)]
pub enum PackageParsingError {
    MissingProjectNameSeed,
    EmptyProjectNameSeed,

    MissingVersion,
    EmptyVersion,

    MissingVisibleName,
    EmptyVisibleName,

    MissingPackageNames,
    EmptySrcName,
    EmptyBinName,
}

impl std::error::Error for PackageParsingError {}
