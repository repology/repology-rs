// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Serialize;
use strum::{AsRefStr, FromRepr};

#[derive(
    Debug,
    PartialEq,
    Serialize,
    FromRepr,
    sqlx::Type,
    Clone,
    Copy,
    AsRefStr,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[repr(i16)]
pub enum PackageStatus {
    Newest = 1,
    Outdated = 2,
    Ignored = 3,
    Unique = 4,
    Devel = 5,
    Legacy = 6,
    Incorrect = 7,
    Untrusted = 8,
    NoScheme = 9,
    Rolling = 10,
}
