// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

pub struct WalkEntry {
    pub path_absolute: PathBuf,
    pub path_relative: PathBuf,
}

enum WalkFilter<'a> {
    Name(&'a str),
    NameSuffix(&'a str),
}

pub struct WalkFileTree<'a> {
    path: &'a Path,
    walkdir: <walkdir::WalkDir as IntoIterator>::IntoIter,
    filter: WalkFilter<'a>,
}

impl<'a> WalkFileTree<'a> {
    pub fn walk_by_name(path: &'a Path, name: &'a str) -> Self {
        Self {
            path,
            walkdir: walkdir::WalkDir::new(path).sort_by_file_name().into_iter(),
            filter: WalkFilter::Name(name),
        }
    }

    #[cfg_attr(not(test), expect(unused))] // will be used in parsers
    pub fn walk_by_suffix(path: &'a Path, suffix: &'a str) -> Self {
        Self {
            path,
            walkdir: walkdir::WalkDir::new(path).sort_by_file_name().into_iter(),
            filter: WalkFilter::NameSuffix(suffix),
        }
    }
}

impl Iterator for WalkFileTree<'_> {
    type Item = anyhow::Result<WalkEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry = match self.walkdir.next() {
                Some(Ok(entry)) => entry,
                Some(Err(e)) => {
                    return Some(Err(e.into()));
                }
                None => {
                    return None;
                }
            };

            if entry.file_type().is_file() {
                let Some(file_name) = entry.file_name().to_str() else {
                    continue;
                };

                if !match self.filter {
                    WalkFilter::Name(name) => file_name == name,
                    WalkFilter::NameSuffix(suffix) => file_name.ends_with(suffix),
                } {
                    continue;
                }

                let path_absolute = entry.into_path();
                let path_relative = path_absolute.strip_prefix(self.path).expect("expected to be able to strip path prefix from the child of that very prefix").to_path_buf();
                return Some(Ok(WalkEntry {
                    path_absolute,
                    path_relative,
                }));
            }
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    #[test]
    fn test_by_name() {
        let dir = TempDir::new().unwrap();

        {
            let subdir = dir.path().join("foo");
            std::fs::create_dir(&subdir).unwrap();
            std::fs::File::create(subdir.join("PKGBUILD")).unwrap();
            // should not return this file, as it's named differently
            std::fs::File::create(subdir.join("Makefile")).unwrap();

            let subdir = dir.path().join("bar");
            std::fs::create_dir(&subdir).unwrap();
            std::fs::File::create(subdir.join("PKGBUILD")).unwrap();

            // should not return this, as it's a directory
            std::fs::create_dir(dir.path().join("PKGBUILD")).unwrap();
        }

        let mut res = vec![];
        for entry in WalkFileTree::walk_by_name(dir.path(), "PKGBUILD") {
            let entry = entry.unwrap();
            res.push(entry.path_relative);
        }
        assert_eq!(
            res,
            vec![Path::new("bar/PKGBUILD"), Path::new("foo/PKGBUILD")]
        );
    }

    #[test]
    fn test_by_suffix() {
        let dir = TempDir::new().unwrap();

        {
            let subdir = dir.path().join("foo");
            std::fs::create_dir(&subdir).unwrap();
            std::fs::File::create(subdir.join("foo.spec")).unwrap();
            // should not return this file, as it's named differently
            std::fs::File::create(subdir.join("foo.conf")).unwrap();

            let subdir = dir.path().join("bar");
            std::fs::create_dir(&subdir).unwrap();
            std::fs::File::create(subdir.join("bar.spec")).unwrap();

            // should not return this, as it's a directory
            std::fs::create_dir(dir.path().join("baz.spec")).unwrap();
        }

        let mut res = vec![];
        for entry in WalkFileTree::walk_by_suffix(dir.path(), ".spec") {
            let entry = entry.unwrap();
            res.push(entry.path_relative);
        }
        assert_eq!(
            res,
            vec![Path::new("bar/bar.spec"), Path::new("foo/foo.spec")]
        );
    }
}
