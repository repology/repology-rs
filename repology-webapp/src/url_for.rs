// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use indexmap::IndexMap;

use anyhow::{Result, bail};

pub struct UrlConstructor<'a> {
    pattern: &'static str,
    fields: IndexMap<&'a str, &'a str>,
}

impl<'a> UrlConstructor<'a> {
    pub fn new(pattern: &'static str) -> Self {
        Self {
            pattern,
            fields: Default::default(),
        }
    }

    pub fn construct(&self) -> Result<String> {
        let mut fields = self.fields.clone();

        let mut res = String::new();

        let mut first = true;
        for segment in self.pattern.split('/') {
            if !first {
                res += "/";
            }
            first = false;

            if let Some(full_field_name) = segment.strip_prefix(':') {
                let (field_name_without_ext, extension) = full_field_name
                    .split_at(full_field_name.find('.').unwrap_or(full_field_name.len()));

                // XXX: field names may currently come from either
                // - axum Path extractor (in which case they include extension for
                //   components like :foo.txt), until #31 is implemented
                // - manually specified params in e.g. url_for in templates (in
                //   which case we don't want to specify any extensions so we don't)
                // Until #31 is implemented, we have to support both.
                if let Some(field_value) = fields.shift_remove(&full_field_name) {
                    // TODO: remove this branch when issue #31 is fixed
                    res += &url_escape::encode_component(field_value);
                } else if let Some(field_value) = fields.shift_remove(&field_name_without_ext) {
                    res += &url_escape::encode_component(field_value);
                    if !extension.is_empty() {
                        res += &url_escape::encode_component(extension);
                    }
                } else {
                    bail!(
                        "missing required field {} when trying to construct url for {} with {:?}",
                        full_field_name,
                        self.pattern,
                        self.fields
                    );
                }
            } else if let Some(field_name) = segment.strip_prefix('*') {
                if let Some(field_value) = fields.shift_remove(&field_name) {
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
                // since this comes from path, assumed to be valid path part with no need to escape
                res += segment;
            }
        }

        // repeat for new-style {} placeholders
        let mut pattern = res.as_str();
        let mut res = String::new();

        loop {
            let Some((prefix, rest)) = pattern.split_once('{') else {
                res += pattern;
                break;
            };

            res += prefix;

            let Some((field_name, rest)) = rest.split_once('}') else {
                bail!("missing closing brace in pattern {}", self.pattern);
            };

            let (field_name, is_path) = if let Some(field_name) = field_name.strip_prefix('*') {
                (field_name, true)
            } else {
                (field_name, false)
            };

            if let Some(field_value) = fields.shift_remove(&field_name) {
                if is_path {
                    res += &url_escape::encode_path(field_value)
                } else {
                    res += &url_escape::encode_component(field_value)
                }
            } else {
                bail!(
                    "missing required field {} when trying to construct url for {} with {:?}",
                    field_name,
                    self.pattern,
                    self.fields
                );
            }

            pattern = rest;
        }

        let fragment = fields.shift_remove("_fragment");
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
                self.fields.shift_remove(key);
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
            "/aaa/bbb/ccc/ddd?e=eee&g=ggg#fff"
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
    fn test_url_constructor_query_param_order() {
        let mut c = UrlConstructor::new("/");

        c.add_fields([("b", "b"), ("a", "a")]);
        c.add_fields([("d", "d"), ("c", "c")]);
        assert_eq!(c.construct().unwrap(), "/?b=b&a=a&d=d&c=c");
        c.add_fields([("a", "")]);
        assert_eq!(c.construct().unwrap(), "/?b=b&d=d&c=c");
        c.add_fields([("d", "")]);
        assert_eq!(c.construct().unwrap(), "/?b=b&c=c");
        c.add_fields([("b", "")]);
        assert_eq!(c.construct().unwrap(), "/?c=c");
    }

    #[test]
    fn test_url_constructor_wildcard() {
        let mut c = UrlConstructor::new("/:a/bbb/*c");

        c.add_fields([("a", "aaa")]);
        c.add_fields([("c", "ccc/ddd")]);
        c.add_fields([("e", "eee")]);
        assert_eq!(c.construct().unwrap(), "/aaa/bbb/ccc/ddd?e=eee");
    }

    #[test]
    fn test_url_constructor_extension() {
        let mut c = UrlConstructor::new("/:filename.txt");

        c.add_fields([("filename", "foo")]);
        assert_eq!(c.construct().unwrap(), "/foo.txt");
    }

    #[test]
    fn test_url_constructor_extension_legacy() {
        // TODO: remove this test when issue #31 is fixed
        let mut c = UrlConstructor::new("/:filename.txt");

        c.add_fields([("filename.txt", "foo.txt")]);
        assert_eq!(c.construct().unwrap(), "/foo.txt");
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests_new_axum {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_url_constructor() {
        let mut c = UrlConstructor::new("/{a}/{b}/ccc/{d}");

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
            "/aaa/bbb/ccc/ddd?e=eee&g=ggg#fff"
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
    fn test_url_constructor_query_param_order() {
        let mut c = UrlConstructor::new("/");

        c.add_fields([("b", "b"), ("a", "a")]);
        c.add_fields([("d", "d"), ("c", "c")]);
        assert_eq!(c.construct().unwrap(), "/?b=b&a=a&d=d&c=c");
        c.add_fields([("a", "")]);
        assert_eq!(c.construct().unwrap(), "/?b=b&d=d&c=c");
        c.add_fields([("d", "")]);
        assert_eq!(c.construct().unwrap(), "/?b=b&c=c");
        c.add_fields([("b", "")]);
        assert_eq!(c.construct().unwrap(), "/?c=c");
    }

    #[test]
    fn test_url_constructor_wildcard() {
        let mut c = UrlConstructor::new("/{a}/bbb/{*c}");

        c.add_fields([("a", "aaa")]);
        c.add_fields([("c", "ccc/ddd")]);
        c.add_fields([("e", "eee")]);
        assert_eq!(c.construct().unwrap(), "/aaa/bbb/ccc/ddd?e=eee");
    }

    #[test]
    fn test_url_constructor_extension() {
        let mut c = UrlConstructor::new("/{filename}.txt");

        c.add_fields([("filename", "foo")]);
        assert_eq!(c.construct().unwrap(), "/foo.txt");
    }
}
