// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use anyhow::Error;
use tracing::{error, info};

#[allow(unused)]
pub enum FontStyle {
    Regular,
    Bold,
}

struct Font {
    ttf_data: Vec<u8>,
}

// TODO: there are many options of handling these fonts, for instance
// we count include these into binary with include_bytes!
// Alternatively, we could precalculate advances for all expected glyphs
// (ASCII + …?) and shore these instead of a font
const FONT_DIRS: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu", // Ubuntu
    "/usr/local/share/fonts/dejavu",    // FreeBSD
];

impl Font {
    pub fn new(file_name: &str) -> Self {
        for dir in FONT_DIRS {
            let path = Path::new(dir).join(file_name);
            if let Ok(ttf_data) = std::fs::read(&path) {
                info!(file_name, ?path, "discovered true type font {}", file_name);
                return Self { ttf_data };
            }
        }
        error!(file_name, dirs_tried = ?FONT_DIRS, "failed to find font {}", file_name);
        panic!(
            "Font {} not found in known font directories {:?}",
            file_name, FONT_DIRS
        );
    }
}

pub struct FontMeasurer {
    regular: Font,
    bold: Font,
}

impl FontMeasurer {
    pub fn new() -> Self {
        Self {
            regular: Font::new("DejaVuSans.ttf"),
            bold: Font::new("DejaVuSans-Bold.ttf"),
        }
    }

    pub fn get_text_width(
        &self,
        text: &str,
        size: usize,
        style: FontStyle,
    ) -> Result<usize, Error> {
        // as ttf_parser documentation say, Face::parse is really fast so there's no
        // need to store it in Font
        let face = ttf_parser::Face::parse(
            match style {
                FontStyle::Regular => &self.regular.ttf_data,
                FontStyle::Bold => &self.bold.ttf_data,
            },
            0,
        )?;

        Ok(text
            .chars()
            .map(|ch| {
                face.glyph_index(ch)
                    .and_then(|glyph_id| face.glyph_hor_advance(glyph_id))
                    .unwrap_or(0) as f32
                    * size as f32
                    / face.units_per_em() as f32
            })
            .sum::<f32>() as usize)
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use super::*;

    #[test]
    fn font_measurer() {
        let font_measurer = FontMeasurer::new();

        assert!(
            font_measurer
                .get_text_width("foo", 10, FontStyle::Regular)
                .unwrap()
                > 0
        );
        assert!(
            font_measurer
                .get_text_width("foo", 20, FontStyle::Regular)
                .unwrap()
                > font_measurer
                    .get_text_width("foo", 10, FontStyle::Regular)
                    .unwrap()
        );
    }
}
