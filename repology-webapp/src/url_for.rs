// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use anyhow::{bail, Error};

pub struct UrlConstructor<'a> {
    pattern: &'static str,
    fields: HashMap<&'a str, &'a str>,
}

impl<'a> UrlConstructor<'a> {
    pub fn new(pattern: &'static str) -> Self {
        Self {
            pattern,
            fields: Default::default(),
        }
    }

    pub fn construct(&self) -> Result<String, Error> {
        let mut fields = self.fields.clone();

        let mut res = String::new();

        let mut first = true;
        for segment in self.pattern.split('/') {
            if !first {
                res += "/";
            }
            first = false;

            if let Some(field_name) = segment.strip_prefix(':') {
                if let Some(field_value) = fields.remove(&field_name) {
                    res += &url_escape::encode_component(field_value);
                } else {
                    bail!(
                        "missing required field {} when trying to construct url for {} with {:?}",
                        field_name,
                        self.pattern,
                        self.fields
                    );
                }
            } else if let Some(field_name) = segment.strip_prefix('*') {
                if let Some(field_value) = fields.remove(&field_name) {
                    res += &url_escape::encode_path(field_value);
                } else {
                    bail!(
                        "missing required field {} when trying to construct url for {} with {:?}",
                        field_name,
                        self.pattern,
                        self.fields
                    );
                }
            } else {
                // since this comes from path, assumed to be valid path part
                res += segment;
            }
        }

        let fragment = fields.remove("_fragment");
        let mut first = true;
        for (key, value) in fields {
            res += if first { "?" } else { "&" };
            first = false;
            res += &url_escape::encode_component(key);
            res += "=";
            res += &url_escape::encode_component(value);
        }

        if let Some(fragment) = fragment {
            res += "#";
            res += &url_escape::encode_component(fragment);
        }

        Ok(res)
    }

    pub fn add_fields<T>(&mut self, fields: T)
    where
        T: IntoIterator<Item = (&'a str, &'a str)>,
    {
        fields.into_iter().for_each(|(key, value)| {
            if value.is_empty() {
                self.fields.remove(key);
            } else {
                self.fields.insert(key, value);
            }
        });
    }

    pub fn with_fields(mut self, fields: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        self.add_fields(fields);
        self
    }

    pub fn with_field(mut self, key: &'a str, value: &'a str) -> Self {
        self.fields.insert(key, value);
        self
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_url_constructor() {
        let mut c = UrlConstructor::new("/:a/:b/ccc/:d");

        assert_matches!(c.construct(), Err(_));

        c.add_fields([("a", "aaa")]);
        assert_matches!(c.construct(), Err(_));

        c.add_fields([("b", "bbb"), ("d", "ddd")]);
        assert_eq!(c.construct().unwrap(), "/aaa/bbb/ccc/ddd");

        c.add_fields([("e", "eee")]);
        assert_eq!(c.construct().unwrap(), "/aaa/bbb/ccc/ddd?e=eee");

        c.add_fields([("_fragment", "fff")]);
        assert_eq!(c.construct().unwrap(), "/aaa/bbb/ccc/ddd?e=eee#fff");

        c.add_fields([("g", "ggg")]);
        assert_matches!(
            c.construct().unwrap().as_ref(),
            // query param order is undefined because of HashMap
            "/aaa/bbb/ccc/ddd?e=eee&g=ggg#fff" | "/aaa/bbb/ccc/ddd?g=ggg&e=eee#fff"
        );

        c.add_fields([("e", ""), ("g", "")]);
        assert_eq!(c.construct().unwrap(), "/aaa/bbb/ccc/ddd#fff");

        c.add_fields([("a", "")]);
        assert_matches!(c.construct(), Err(_));

        let escapable = "_/?#%_";
        c.add_fields([("a", escapable), ("e", escapable), ("_fragment", escapable)]);
        assert_eq!(
            c.construct().unwrap(),
            "/_%2F%3F%23%25_/bbb/ccc/ddd?e=_%2F%3F%23%25_#_%2F%3F%23%25_"
        );
    }

    #[test]
    fn test_url_constructor_wildcard() {
        let mut c = UrlConstructor::new("/:a/bbb/*c");

        c.add_fields([("a", "aaa")]);
        c.add_fields([("c", "ccc/ddd")]);
        c.add_fields([("e", "eee")]);
        assert_eq!(c.construct().unwrap(), "/aaa/bbb/ccc/ddd?e=eee");
    }
}
