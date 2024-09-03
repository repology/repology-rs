// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt;

use serde::de::{Deserializer, Error, Visitor};

struct BoolFromStrVisitor;

impl<'de> Visitor<'de> for BoolFromStrVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string value denoting true if non-empty and false otherwise")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(!value.is_empty())
    }
}

// required for backward compatibility with Python/Flask code, where boolean query
// flags were interpreted as bool(str), which treats any non-empty value as true
// and empty string as false
pub fn deserialize_bool_flag<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let res = deserializer.deserialize_str(BoolFromStrVisitor);
    res
}
