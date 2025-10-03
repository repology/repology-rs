// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;

use repology_common::PackageStatus;

use crate::font::{FontMeasurer, FontStyle};
use crate::xmlwriter::{XmlTag, xml};

const HEADER_HEIGHT: usize = 25;
const HEADER_FONT_SIZE: usize = 15;
const CELL_HEIGHT: usize = 20;
const CELL_FONT_SIZE: usize = 11;
const CELL_HORIZONTAL_PADDING: usize = 5;

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
pub struct Cell {
    // TODO: switch to Cow here to avoid allocation
    pub text: String,
    pub clazz: Option<String>,
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

    pub fn clazz(mut self, clazz: &str) -> Self {
        self.clazz = Some(clazz.to_owned());
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
                    font_measurer.get_text_width(&cell.text, CELL_FONT_SIZE, FontStyle::Regular)?
                        + CELL_HORIZONTAL_PADDING * 2,
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
    let (min_width, header_height) =
        if let Some(header) = header.filter(|header| !header.is_empty()) {
            (
                min_width.max(
                    font_measurer.get_text_width(header, HEADER_FONT_SIZE, FontStyle::Bold)?
                        + CELL_HORIZONTAL_PADDING * 2,
                ),
                HEADER_HEIGHT,
            )
        } else {
            (min_width, 0)
        };

    // calculate total sizes
    let total_height = header_height + CELL_HEIGHT * num_rows;
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
    doc.add_child(
        xml!("clipPath", "id" = "clip").with_child(
            xml!("rect", "rx" = 3, "width" = "100%", "height" = "100%", "class" = "clip"),
        ),
    );

    // define linear gradient for bevel effect
    doc.add_child(
        xml!("linearGradient", "id" = "grad", "x2" = 0, "y2" = "100%")
            .with_child(xml!("stop", "offset" = 0, "stop-opacity" = ".1"))
            .with_child(xml!("stop", "offset" = 1, "stop-opacity" = ".1")),
    );

    // TODO: use const_format?
    let style = format!("\
        g{{\
            fill:#fff;font-family:DejaVu Sans,Verdana,Geneva,sans-serif;\
            font-size:{CELL_FONT_SIZE}px\
        }}\
        .bg{{fill:#555}}\
        .clip{{fill:#000}}\
        .gradient_tone{{stop-color:#bbb}}\
        .row_bg{{width:100%;height:{CELL_HEIGHT}px;fill:url(#grad)}}\
        .left,.center,.right,.title{{dominant-baseline:central}}\
        .title{{text-anchor:middle;font-weight:bold;font-size:15px}}\
        .left{{text-anchor:start}}\
        .center{{text-anchor:middle}}\
        .right{{text-anchor:end}}\
        .outline{{fill:#010101;fill-opacity:.3}}\
        .special{{fill:#e00000}}\
        .outdated,.legacy{{fill:#e05d44}}\
        .newest,.unique,.devel{{fill:#4c1}}\
        .other{{fill:#9f9f9f}}\
        .nice{{fill:#007ec6}}\
    ");
    doc.add_child(xml!("style").with_text(&style));

    // graphical data
    let mut g = xml!("g", "clip-path" = "url(#clip)");

    // background
    g.add_child(xml!("rect", "width" = "100%", "height" = "100%", "class" = "bg"));

    // header
    if let Some(header) = header {
        g.add_child(
            xml!("g")
                .with_child(
                    xml!("text", "x" = total_width / 2, "y" = HEADER_HEIGHT / 2 + 1, "class" = "title outline")
                        .with_text(header),
                )
                .with_child(
                    xml!("text", "x" = total_width / 2, "y" = HEADER_HEIGHT / 2, "class" = "title")
                        .with_text(header),
                )
        );
    }

    // rows
    for (nrow, row) in cells.iter().enumerate() {
        let row_y_offset = header_height + nrow * CELL_HEIGHT;

        // cell backgrounds
        for (cell, column) in row.iter().zip(columns.iter()) {
            if let Some(clazz) = &cell.clazz {
                g.add_child(
                    xml!("rect", "x" = column.offset, "y" = row_y_offset, "width" = column.width, "height" = CELL_HEIGHT, "class" = clazz)
                );
            }
        }

        // gradient
        g.add_child(
            xml!("rect", "y" = row_y_offset, "width" = "100%", "height" = CELL_HEIGHT, "class" = "row_bg")
        );

        // cell texts
        let mut cell_text_g = xml!("g");

        for (cell, column) in row.iter().zip(columns.iter()) {
            if cell.text.is_empty() || column.is_collapsed {
                continue;
            }

            let (text_x, text_class) = match cell.alignment {
                CellAlignment::Left => (column.offset + CELL_HORIZONTAL_PADDING, "left"),
                CellAlignment::Center => (column.offset + column.width / 2, "center"),
                CellAlignment::Right => (
                    column.offset + column.width - CELL_HORIZONTAL_PADDING,
                    "right",
                ),
            };

            let text_y = row_y_offset + CELL_HEIGHT / 2;
            cell_text_g.add_child(
                xml!("text", "x" = text_x, "y" = text_y + 1, "class" = format!("{text_class} outline"))
                    .with_text(&cell.text)
            );

            cell_text_g.add_child(
                xml!("text", "x" = text_x, "y" = text_y, "class" = text_class)
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
pub fn badge_clazz_for_package_status(
    package_status: PackageStatus,
    special_status: Option<SpecialVersionStatus>,
) -> &'static str {
    if let Some(special_status) = special_status {
        use SpecialVersionStatus::*;
        match special_status {
            LowerThanUserGivenThreshold => "special",
        }
    } else {
        use PackageStatus::*;
        match package_status {
            Outdated => "outdated",
            Legacy => "legacy",
            Newest => "newest",
            Unique => "unique",
            Devel => "devel",
            _ => "other",
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
