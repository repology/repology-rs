// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;

use repology_common::PackageStatus;

use crate::font::{FontMeasurer, FontStyle};
use crate::xmlwriter::{XmlTag, xml};

// visual length is implied here, as we use it to account for ellipsis in truncated string
const ELLIPSIS_VISUAL_WIDTH_CHARS: usize = 1;

#[derive(Default)]
#[expect(dead_code)]
pub enum CellAlignment {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Default)]
pub struct Theme {
    pub background: &'static str,
    pub cell_font_size: usize,
    pub cell_height: usize,
    pub cell_horizontal_padding: usize,
    pub clip_fill: &'static str,
    pub color_devel: &'static str,
    pub color_legacy: &'static str,
    pub color_newest: &'static str,
    pub color_nice: &'static str,
    pub color_other: &'static str,
    pub color_outdated: &'static str,
    pub color_special: &'static str,
    pub color_unique: &'static str,
    pub font: &'static str,
    pub gradient_tone: &'static str,
    pub header_font_size: usize,
    pub header_height: usize,
    pub outline_color: &'static str,
    pub outline_opacity: &'static str,
    pub text_color: &'static str,
}

pub static DEFAULT_THEME: Theme = Theme {
    background: "#555",
    cell_font_size: 11,
    cell_height: 20,
    cell_horizontal_padding: 5,
    clip_fill: "#000",
    color_devel: "#4c1",
    color_legacy: "#e05d44",
    color_newest: "#4c1",
    color_nice: "#007ec6",
    color_other: "#9f9f9f",
    color_outdated: "#e05d44",
    color_special: "#e00000",
    color_unique: "#4c1",
    font: "DejaVu Sans,Verdana,Geneva,sans-serif",
    gradient_tone: "#bbb",
    header_font_size: 15,
    header_height: 25,
    outline_color: "#010101",
    outline_opacity: ".3",
    text_color: "#fff",
};

#[derive(Default)]
pub struct Cell {
    // TODO: switch to Cow here to avoid allocation
    pub text: String,
    pub color: Option<String>,
    pub min_width: usize,
    pub collapsible: bool,
    pub alignment: CellAlignment,
}

impl Cell {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            ..Default::default()
        }
    }

    pub fn empty() -> Self {
        Self {
            text: String::new(),
            ..Default::default()
        }
    }

    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.to_owned());
        self
    }

    pub fn truncate(mut self, length: usize) -> Self {
        let char_len = self.text.chars().count();
        if char_len <= length {
            return self;
        }
        for (char_pos, (byte_pos, _)) in self.text.char_indices().enumerate() {
            if char_pos + ELLIPSIS_VISUAL_WIDTH_CHARS >= length {
                self.text.truncate(byte_pos);
                while self.text.ends_with('.') {
                    self.text.pop();
                }
                self.text += "…";
                return self;
            }
        }
        self
    }

    pub fn align(mut self, alignment: CellAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn min_width(mut self, width: usize) -> Self {
        self.min_width = width;
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }
}

#[derive(Clone, Default)]
struct Column {
    pub width: usize = 0,
    pub is_collapsed: bool = true,
    pub offset: usize = 0,
}

#[rustfmt::skip::macros(xml)]
pub fn render_generic_badge(
    cells: &[Vec<Cell>],
    header: Option<&str>,
    min_width: usize,
    font_measurer: &FontMeasurer,
    theme: &Theme,
) -> Result<String> {
    let num_rows = cells.len();
    let num_columns = if num_rows > 0 { cells[0].len() } else { 0 };

    let mut columns = vec![Column::default(); num_columns];

    // calculate column widths
    for row in cells {
        for (cell, column) in row.iter().zip(columns.iter_mut()) {
            column.width = column
                .width
                .max(
                    font_measurer.get_text_width(
                        &cell.text,
                        theme.cell_font_size,
                        FontStyle::Regular,
                    )? + theme.cell_horizontal_padding * 2,
                )
                .max(cell.min_width);
            if !cell.text.is_empty() || !cell.collapsible {
                column.is_collapsed = false;
            }
        }
    }

    // handle collapsed columns
    columns
        .iter_mut()
        .filter(|column| column.is_collapsed)
        .for_each(|column| column.width = 0);

    // add header if specified
    let (min_width, header_height) = if let Some(header) =
        header.filter(|header| !header.is_empty())
    {
        (
            min_width.max(
                font_measurer.get_text_width(header, theme.header_font_size, FontStyle::Bold)?
                    + theme.cell_horizontal_padding * 2,
            ),
            theme.header_height,
        )
    } else {
        (min_width, 0)
    };

    // calculate total sizes
    let total_height = header_height + theme.cell_height * num_rows;
    let mut total_width = columns.iter().map(|column| column.width).sum::<usize>();

    // force minimal width by expanding leftmost column
    if total_width < min_width {
        if let Some(first_column) = columns.first_mut() {
            first_column.width += min_width - total_width;
        }
        total_width = min_width;
    }

    // calculate column offsets
    let mut offset = 0;
    columns.iter_mut().for_each(|column| {
        column.offset = offset;
        offset += column.width;
    });

    // start SVG document
    let mut doc = xml!("svg", "xmlns" = "http://www.w3.org/2000/svg", "width" = total_width, "height" = total_height);

    // define clip path for rounded corners
    doc.add_child(xml!("clipPath", "id" = "clip").with_child(
        xml!("rect", "rx" = 3, "width" = "100%", "height" = "100%", "fill" = theme.clip_fill),
    ));

    // define linear gradient for bevel effect
    doc.add_child(
        xml!("linearGradient", "id" = "grad", "x2" = 0, "y2" = "100%")
            .with_child(xml!("stop", "offset" = 0, "stop-color" = theme.gradient_tone, "stop-opacity" = ".1"))
            .with_child(xml!("stop", "offset" = 1, "stop-opacity" = ".1")),
    );

    // graphical data
    let mut g = xml!("g", "clip-path" = "url(#clip)");

    // background
    g.add_child(xml!("rect", "width" = "100%", "height" = "100%", "fill" = theme.background));

    // header
    if let Some(header) = header {
        g.add_child(
            xml!("g", "fill" = theme.text_color, "text-anchor" = "middle", "font-family" = theme.font, "font-size" = theme.header_font_size, "font-weight" = "bold")
                .with_child(
                    xml!("text", "x" = total_width / 2, "y" = theme.header_height / 2 + 1, "dominant-baseline" = "central", "fill" = theme.outline_color, "fill-opacity" = theme.outline_opacity)
                        .with_text(header),
                )
                .with_child(
                    xml!("text", "x" = total_width / 2, "y" = theme.header_height / 2, "dominant-baseline" = "central")
                        .with_text(header),
                )
        );
    }

    // rows
    for (nrow, row) in cells.iter().enumerate() {
        let row_y_offset = header_height + nrow * theme.cell_height;

        // cell backgrounds
        for (cell, column) in row.iter().zip(columns.iter()) {
            if let Some(color) = &cell.color {
                g.add_child(
                    xml!("rect", "x" = column.offset, "y" = row_y_offset, "width" = column.width, "height" = theme.cell_height, "fill" = color)
                );
            }
        }

        // gradient
        g.add_child(
            xml!("rect", "y" = row_y_offset, "width" = "100%", "height" = theme.cell_height, "fill" = "url(#grad)")
        );

        // cell texts
        let mut cell_text_g =
            xml!("g", "fill" = "#fff", "font-family" = theme.font, "font-size" = theme.cell_font_size);

        for (cell, column) in row.iter().zip(columns.iter()) {
            if cell.text.is_empty() || column.is_collapsed {
                continue;
            }

            let (text_x, text_anchor) = match cell.alignment {
                CellAlignment::Left => (column.offset + theme.cell_horizontal_padding, "start"),
                CellAlignment::Center => (column.offset + column.width / 2, "middle"),
                CellAlignment::Right => (
                    column.offset + column.width - theme.cell_horizontal_padding,
                    "end",
                ),
            };

            let text_y = row_y_offset + theme.cell_height / 2;
            cell_text_g.add_child(
                xml!("text", "x" = text_x, "y" = text_y + 1, "text-anchor" = text_anchor, "dominant-baseline" = "central", "fill" = theme.outline_color, "fill-opacity" = theme.outline_opacity)
                    .with_text(&cell.text)
            );

            cell_text_g.add_child(
                xml!("text", "x" = text_x, "y" = text_y, "text-anchor" = text_anchor, "dominant-baseline" = "central")
                    .with_text(&cell.text)
            );
        }

        g.add_child(cell_text_g);
    }

    doc.add_child(g);

    Ok(doc.to_string())
}

#[derive(PartialEq)]
pub enum SpecialVersionStatus {
    LowerThanUserGivenThreshold,
}

#[allow(unused)]
pub fn badge_color_for_package_status(
    package_status: PackageStatus,
    theme: &Theme,
    special_status: Option<SpecialVersionStatus>,
) -> &'static str {
    if let Some(special_status) = special_status {
        use SpecialVersionStatus::*;
        match special_status {
            LowerThanUserGivenThreshold => theme.color_special,
        }
    } else {
        use PackageStatus::*;
        match package_status {
            Outdated => theme.color_outdated,
            Legacy => theme.color_legacy,
            Newest => theme.color_newest,
            Unique => theme.color_unique,
            Devel => theme.color_devel,
            _ => theme.color_other,
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_truncate() {
        // will need update if ELLIPSIS_VISUAL_WIDTH_CHARS is changed
        assert_eq!(Cell::new("abcde").truncate(0).text, "…");
        assert_eq!(Cell::new("abcde").truncate(1).text, "…");
        assert_eq!(Cell::new("abcde").truncate(2).text, "a…");
        assert_eq!(Cell::new("abcde").truncate(3).text, "ab…");
        assert_eq!(Cell::new("abcde").truncate(4).text, "abc…");
        assert_eq!(Cell::new("abcde").truncate(5).text, "abcde");
        assert_eq!(Cell::new("abcde").truncate(6).text, "abcde");
    }

    #[test]
    fn test_cell_truncate_dot() {
        // don't leave handing dot before ellipsis
        assert_eq!(
            Cell::new("ab.cdefg")
                .truncate(3 + ELLIPSIS_VISUAL_WIDTH_CHARS)
                .text,
            "ab…"
        );
    }

    #[test]
    fn test_cell_truncate_unicode() {
        // make sure we are unicode aware and don't truncate mid-codepoint
        assert_eq!(
            Cell::new("абвгдеж")
                .truncate(3 + ELLIPSIS_VISUAL_WIDTH_CHARS)
                .text,
            "абв…"
        );
    }
}
