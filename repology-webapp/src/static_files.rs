// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::LazyLock;

use flate2::{write::GzEncoder, Compression};
use include_dir::{include_dir, Dir, DirEntry};

static STATIC_FILES_RAW: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

pub static STATIC_FILES: LazyLock<StaticFiles> =
    LazyLock::new(|| StaticFiles::new(&STATIC_FILES_RAW));

pub struct StaticFile {
    pub name: &'static str,
    pub hashed_name: String,
    pub original_content: &'static [u8],
    pub compressed_content: Vec<u8>,
}

pub struct StaticFiles {
    by_hashed_name: HashMap<String, StaticFile>,
    hashed_name_by_original_name: HashMap<&'static str, String>,
}

unsafe impl Send for StaticFiles {}

impl StaticFiles {
    pub fn new(dir: &'static Dir) -> Self {
        let static_files_iterator = dir
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
            });

        let by_hashed_name: HashMap<_, _> = static_files_iterator
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
            .collect();

        let hashed_name_by_original_name = by_hashed_name
            .values()
            .map(|file| (file.name, file.hashed_name.clone()))
            .collect();

        Self {
            by_hashed_name,
            hashed_name_by_original_name,
        }
    }

    pub fn by_hashed_name(&self, hashed_name: &str) -> Option<&StaticFile> {
        self.by_hashed_name.get(hashed_name)
    }

    pub fn hashed_name_by_orig_name(&self, orig_name: &str) -> Option<&str> {
        self.hashed_name_by_original_name
            .get(orig_name)
            .map(|name| name.as_ref())
    }
}
