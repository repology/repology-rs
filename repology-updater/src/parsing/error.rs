// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, strum::Display)]
pub enum PackageParsingError {
    MissingProjectNameSeed,
    MissingVersion,
    EmptyProjectNameSeed,
    EmptyVersion,
    //UnexpectedDataFormat,
    //UnexpectedDataFormatString(String),
}

impl std::error::Error for PackageParsingError {}
