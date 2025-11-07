// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::bail;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum Compression {
    Gz,
    Xz,
    Bz2,
    Zstd,
}

impl Compression {
    pub fn from_extension(
        path_or_url: &str,
        base_suffix: &str,
    ) -> Result<Option<Self>, anyhow::Error> {
        if let Some((_, rhs)) = path_or_url.rsplit_once(base_suffix) {
            match rhs {
                "" => Ok(None),
                ".gz" => Ok(Some(Self::Gz)),
                ".bz2" => Ok(Some(Self::Bz2)),
                ".xz" => Ok(Some(Self::Xz)),
                ".zst" => Ok(Some(Self::Zstd)),
                _ => {
                    bail!(
                        "cannot determine compression from file name \"{}\": unknown extension \"{}\"",
                        path_or_url,
                        rhs
                    );
                }
            }
        } else {
            bail!(
                "cannot determine compression from file name \"{}\": missing expected suffix \"{}\"",
                path_or_url,
                base_suffix
            );
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_from_extension() {
        assert_matches!(
            Compression::from_extension("/path/to/file.tar", ".tar"),
            Ok(None)
        );
        assert_matches!(
            Compression::from_extension("/path/to/file.tar.gz", ".tar"),
            Ok(Some(Compression::Gz))
        );
        assert!(Compression::from_extension("/path/to/file.tar.ace", ".tar").is_err());
        assert!(Compression::from_extension("/path/to/file.gz", ".tar").is_err());
    }
}
