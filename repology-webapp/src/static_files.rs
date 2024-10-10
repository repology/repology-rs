// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use flate2::{write::GzEncoder, Compression};
use include_dir::{include_dir, Dir, DirEntry};

static STATIC_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

fn iterate_static_files() -> impl Iterator<Item = (&'static str, &'static [u8])> {
    STATIC_FILES
        .find("**/*")
        .expect("file glob should be valid")
        .filter_map(|entry| {
            if let DirEntry::File(file) = entry {
                Some((
                    file.path()
                        .to_str()
                        .expect("static file names should be utf8"),
                    file.contents(),
                ))
            } else {
                None
            }
        })
}

pub struct StaticFile {
    pub name: &'static str,
    pub hashed_name: String,
    pub original_content: &'static [u8],
    pub compressed_content: Vec<u8>,
}

pub struct StaticFiles {
    by_hashed_name: HashMap<String, StaticFile>,
}

unsafe impl Send for StaticFiles {}

impl StaticFiles {
    pub fn new() -> Self {
        Self {
            by_hashed_name: iterate_static_files()
                .map(|(name, content)| {
                    let compressed_content = {
                        use std::io::Write;
                        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
                        encoder
                            .write_all(content)
                            .expect("compression into memory is not expected to fail");
                        encoder
                            .finish()
                            .expect("compression into memory is not expected to fail")
                    };
                    let hash: u64 = cityhasher::hash(content);
                    let (base, ext) = name
                        .rsplit_once('.')
                        .expect("static files should have extensions");
                    let hashed_name = format!("{}.{:016x}.{}", base, hash, ext);

                    let file = StaticFile {
                        name,
                        hashed_name: hashed_name.clone(),
                        original_content: content,
                        compressed_content,
                    };

                    (hashed_name, file)
                })
                .collect(),
        }
    }

    pub fn by_hashed_name(&self, name: &str) -> Option<&StaticFile> {
        self.by_hashed_name.get(name)
    }

    #[expect(dead_code)]
    pub fn name_to_hashed_name_map(&self) -> HashMap<String, String> {
        self.by_hashed_name
            .values()
            .map(|file| (file.name.to_owned(), file.hashed_name.clone()))
            .collect()
    }
}
