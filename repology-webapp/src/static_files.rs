// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::LazyLock;

use flate2::{write::GzEncoder, Compression};
use include_dir::{include_dir, Dir, DirEntry};
use tracing::info;

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
    files: Vec<StaticFile>,
    // TODO: sobjugate self-referential structures and convert to HashMap<&str, &StaticFile>
    by_hashed_name: HashMap<String, usize>,
    by_orig_name: HashMap<String, usize>,
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

        let files: Vec<_> = static_files_iterator
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

                info!(
                    orig_name = file.name,
                    hashed_name = file.hashed_name,
                    orig_size = file.original_content.len(),
                    compressed_size = file.compressed_content.len(),
                    "adding static file {}",
                    file.name,
                );

                file
            })
            .collect();

        Self {
            by_hashed_name: files
                .iter()
                .enumerate()
                .map(|(i, file)| (file.hashed_name.clone(), i))
                .collect(),
            by_orig_name: files
                .iter()
                .enumerate()
                .map(|(i, file)| (file.name.to_string(), i))
                .collect(),
            files,
        }
    }

    #[expect(dead_code)]
    pub fn by_hashed_name(&self, hashed_name: &str) -> Option<&StaticFile> {
        self.by_hashed_name
            .get(hashed_name)
            .map(|i| &self.files[*i])
    }

    pub fn by_orig_name(&self, orig_name: &str) -> Option<&StaticFile> {
        self.by_orig_name.get(orig_name).map(|i| &self.files[*i])
    }

    pub fn by_either_name(&self, name: &str) -> Option<&StaticFile> {
        self.by_hashed_name
            .get(name)
            .or_else(|| self.by_orig_name.get(name))
            .map(|i| &self.files[*i])
    }
}
