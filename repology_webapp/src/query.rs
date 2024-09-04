// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::de::{Deserialize, Deserializer, IntoDeserializer};

// required for backward compatibility with Python/Flask code, where boolean query
// flags were interpreted as bool(str), which treats any non-empty value as true
// and empty string as false
pub fn deserialize_bool_flag<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(!<&str>::deserialize(deserializer)?.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::extract::Query;
    use axum::http::Uri;
    use serde::Deserialize;
    use std::collections::HashSet;

    #[test]
    fn test_bool_flag() {
        #[derive(Deserialize)]
        struct Params {
            #[serde(default)]
            #[serde(deserialize_with = "deserialize_bool_flag")]
            a: bool,
            #[serde(default)]
            #[serde(deserialize_with = "deserialize_bool_flag")]
            b: bool,
            #[serde(default)]
            #[serde(deserialize_with = "deserialize_bool_flag")]
            c: bool,
            #[serde(default)]
            #[serde(deserialize_with = "deserialize_bool_flag")]
            d: bool,
        }

        let uri: Uri = "https://example.com/foo?b&c=&d=1".parse().unwrap();
        let params = Query::<Params>::try_from_uri(&uri).unwrap().0;

        assert_eq!(params.a, false);
        assert_eq!(params.b, false);
        assert_eq!(params.c, false);
        assert_eq!(params.d, true);
    }
}
