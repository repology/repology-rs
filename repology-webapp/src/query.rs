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
    Ok(!<std::borrow::Cow<str>>::deserialize(deserializer)?.is_empty())
}

pub fn deserialize_seq<'de, D, C, I>(deserializer: D) -> Result<C, D::Error>
where
    D: Deserializer<'de>,
    C: FromIterator<I>,
    I: Deserialize<'de>,
{
    <std::borrow::Cow<str>>::deserialize(deserializer)?
        .split(',')
        .map(|s| I::deserialize(s.into_deserializer()))
        .try_collect()
}

#[cfg(test)]
#[coverage(off)]
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
            #[serde(default)]
            #[serde(deserialize_with = "deserialize_bool_flag")]
            e: bool,
        }

        let uri: Uri = "https://example.com/foo?b&c=&d=1&e=+".parse().unwrap();
        let params = Query::<Params>::try_from_uri(&uri).unwrap().0;

        assert_eq!(params.a, false);
        assert_eq!(params.b, false);
        assert_eq!(params.c, false);
        assert_eq!(params.d, true);
        assert_eq!(params.e, true);
    }

    #[test]
    fn test_seq() {
        #[derive(Debug, PartialEq, Eq, Hash, Deserialize)]
        enum Test {
            Foo,
            Bar,
            Baz,
        }

        #[derive(Debug, PartialEq, Deserialize)]
        struct Params {
            #[serde(default)]
            #[serde(deserialize_with = "deserialize_seq")]
            a: Vec<Test>,

            #[serde(default)]
            #[serde(deserialize_with = "deserialize_seq")]
            b: Vec<Test>,

            #[serde(default)]
            #[serde(deserialize_with = "deserialize_seq")]
            c: HashSet<Test>,
        }

        let uri: Uri = "https://example.com/foo?b=Foo,Bar&c=Bar,Baz"
            .parse()
            .unwrap();
        let params = Query::<Params>::try_from_uri(&uri).unwrap().0;

        assert_eq!(params.a, vec![]);
        assert_eq!(params.b, vec![Test::Foo, Test::Bar]);
        assert_eq!(params.c, HashSet::from([Test::Bar, Test::Baz]));
    }
}
