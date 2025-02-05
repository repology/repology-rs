// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use indoc::indoc;
use serde::Deserialize;

use crate::result::EndpointResult;
use crate::state::AppState;
use crate::xmlwriter::{XmlTag, xml};

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    pub xlimit: Option<u32>,
    pub ylimit: Option<u32>,
}

struct Point<'a> {
    x: u32,
    y: u32,
    label: &'a str,
    color: &'a str,
}

fn round_range(value: u32) -> u32 {
    match value {
        0 => 1,
        (1..=10) => value,
        _ => value.next_multiple_of(10_u32.pow(value.ilog10() - 1)),
    }
}

fn render_map(
    points: &[Point<'_>],
    x_label: &str,
    y_label: &str,
    x_unit: &str,
    y_unit: &str,
    x_limit: Option<u32>,
    y_limit: Option<u32>,
) -> String {
    const IMAGE_WIDTH: u32 = 1140;
    const IMAGE_HEIGHT: u32 = 800;
    const MARGIN_TOP: u32 = 30;
    const MARGIN_RIGHT: u32 = 50;
    const MARGIN_BOTTOM: u32 = 20;
    const MARGIN_LEFT: u32 = 40;

    const MAP_WIDTH: u32 = IMAGE_WIDTH - MARGIN_LEFT - MARGIN_RIGHT;
    const MAP_HEIGHT: u32 = IMAGE_HEIGHT - MARGIN_TOP - MARGIN_BOTTOM;

    let mut doc = xml!(
        "svg",
        "xmlns" = "http://www.w3.org/2000/svg",
        "width" = IMAGE_WIDTH,
        "height" = IMAGE_HEIGHT
    );

    let x_max = round_range(points.iter().map(|point| point.x).max().unwrap_or(1));
    let y_max = round_range(points.iter().map(|point| point.y).max().unwrap_or(1));

    let x_max = x_limit
        .filter(|limit| *limit > 0)
        .map_or(x_max, |limit| limit.min(x_max));
    let y_max = y_limit
        .filter(|limit| *limit > 0)
        .map_or(y_max, |limit| limit.min(y_max));

    // define arrow marker
    doc.add_child(
        xml!("defs").with_child(
            xml!(
                "marker",
                "id" = "arrow",
                "markerWidth" = 10,
                "markerHeight" = 10,
                "refX" = 2,
                "refY" = 3,
                "orient" = "auto",
                "markerUnits" = "strokeWidth"
            )
            .with_child(xml!("path", "d" = "M0,0 L2,3 L0,6 L9,3 z", "fill" = "#000",)),
        ),
    );

    // background
    doc.add_child(xml!(
        "rect",
        "width" = IMAGE_WIDTH,
        "height" = IMAGE_HEIGHT,
        "fill" = "#f0f0f0"
    ));

    // dead zone
    {
        const DEAD_ZONE_HEIGHT: u32 = IMAGE_HEIGHT - MARGIN_BOTTOM - MARGIN_TOP / 2;
        let x = MAP_WIDTH as f32 / MAP_HEIGHT as f32 * y_max as f32 / x_max as f32
            * DEAD_ZONE_HEIGHT as f32;

        doc.add_child(xml!(
            "path",
            "d" = format!(
                "M {},{} V {} H {} Z",
                MARGIN_LEFT as f32 + 0.5,
                (IMAGE_HEIGHT - MARGIN_BOTTOM) as f32 + 0.5,
                (MARGIN_TOP / 2) as f32 + 0.5,
                MARGIN_LEFT as f32 + x + 0.5
            ),
            "fill" = "#d0d0d0",
        ));
    }

    // axes
    doc.add_child(xml!(
        "line",
        "marker-end" = "url(#arrow)",
        "x1" = MARGIN_LEFT as f32 + 0.5,
        "x2" = MARGIN_LEFT as f32 + 0.5,
        "y1" = (IMAGE_HEIGHT - MARGIN_BOTTOM) as f32 + 0.5,
        "y2" = (MARGIN_TOP / 2) as f32 + 0.5,
        "stroke" = "#000",
    ));
    doc.add_child(xml!(
        "line",
        "marker-end" = "url(#arrow)",
        "x1" = MARGIN_LEFT as f32 + 0.5,
        "x2" = (IMAGE_WIDTH - MARGIN_RIGHT / 2) as f32 + 0.5,
        "y1" = (IMAGE_HEIGHT - MARGIN_BOTTOM) as f32 + 0.5,
        "y2" = (IMAGE_HEIGHT - MARGIN_BOTTOM) as f32 + 0.5,
        "stroke" = "#000",
    ));

    // axes labels
    doc.add_child(
        xml!(
            "g",
            "fill" = "#000",
            "font-family" = "DejaVu Sans,Verdana,Geneva,sans-serif",
            "font-size" = 13,
            "font-weight" = "bold",
            "text-anchor" = "middle"
        )
        .with_child(
            xml!(
                "text",
                "x" = (MARGIN_LEFT + MAP_WIDTH / 2) as f32 + 0.5,
                "y" = (IMAGE_HEIGHT - MARGIN_BOTTOM / 2 + 3) as f32 + 0.5
            )
            .with_text(x_label),
        )
        .with_child(
            xml!(
                "text",
                "x" = 0,
                "y" = 0,
                "transform" = &format!(
                    "translate({},{}),rotate(-90)",
                    (MARGIN_LEFT / 2) as f32 + 0.5,
                    (MARGIN_TOP + MAP_HEIGHT / 2) as f32 + 0.5
                )
            )
            .with_text(y_label),
        ),
    );

    // axes ticks
    doc.add_child(
        xml!(
            "g",
            "fill" = "#000",
            "font-family" = "DejaVu Sans,Verdana,Geneva,sans-serif",
            "font-size" = 11
        )
        .with_child(
            xml!(
                "text",
                "text-anchor" = "middle",
                "x" = (IMAGE_WIDTH - MARGIN_RIGHT + 3) as f32 + 0.5,
                "y" = (IMAGE_HEIGHT - MARGIN_BOTTOM / 2 + 3) as f32 + 0.5
            )
            .with_text(&format!("{x_max}{x_unit}")),
        )
        .with_child(
            xml!(
                "text",
                "text-anchor" = "middle",
                "x" = MARGIN_LEFT as f32 + 0.5,
                "y" = (IMAGE_HEIGHT - MARGIN_BOTTOM / 2) as f32 + 0.5
            )
            .with_text(&format!("0{x_unit}")),
        )
        .with_child(
            xml!(
                "text",
                "text-anchor" = "end",
                "x" = (MARGIN_LEFT - 5) as f32 + 0.5,
                "y" = (IMAGE_HEIGHT - MARGIN_BOTTOM + 3) as f32 + 0.5
            )
            .with_text(&format!("0{y_unit}")),
        )
        .with_child(
            xml!(
                "text",
                "text-anchor" = "end",
                "x" = (MARGIN_LEFT - 5) as f32 + 0.5,
                "y" = (MARGIN_TOP + 3) as f32 + 0.5
            )
            .with_text(&format!("{y_max}{y_unit}")),
        ),
    );

    // data points
    let pixel_x =
        |x: u32| (MARGIN_LEFT as f32 + x as f32 / x_max as f32 * MAP_WIDTH as f32).floor() + 0.5;
    let pixel_y = |y: u32| {
        ((IMAGE_HEIGHT - MARGIN_BOTTOM) as f32 - y as f32 / y_max as f32 * MAP_HEIGHT as f32)
            .floor()
            + 0.5
    };

    // labels
    for point in points {
        if point.x > x_max || point.y > y_max {
            continue;
        }
        if point.label.is_empty() {
            continue;
        }
        let x = pixel_x(point.x);
        let y = pixel_y(point.y);
        let (text_anchor, x_offset) = if x / (IMAGE_WIDTH as f32) < 0.9 {
            ("start", 5.0)
        } else {
            ("end", -5.0)
        };

        doc.add_child(
            xml!(
                "g",
                "font-family" = "DejaVu Sans,Verdana,Geneva,sans-serif",
                "font-size" = 11,
                "text-anchor" = text_anchor,
            )
            .with_child(
                xml!(
                    "text",
                    "stroke-linecap" = "round",
                    "stroke-width" = 3,
                    "fill" = "#f0f0f0",
                    "stroke" = "#f0f0f0",
                    "x" = x + x_offset,
                    "y" = y + 3.0
                )
                .with_text(point.label),
            )
            .with_child(
                xml!("text", "fill" = "#000", "x" = x + x_offset, "y" = y + 3.0)
                    .with_text(point.label),
            ),
        );
    }

    // points
    for point in points {
        if point.x > x_max || point.y > y_max {
            continue;
        }
        let x = pixel_x(point.x);
        let y = pixel_y(point.y);

        doc.add_child(xml!(
            "circle",
            "cx" = x,
            "cy" = y,
            "r" = 5,
            "fill" = "#f0f0f0",
        ));
        doc.add_child(xml!(
            "circle",
            "cx" = x,
            "cy" = y,
            "r" = 4,
            "fill" = format!("#{}", point.color),
        ));
    }

    doc.to_string()
}

#[cfg_attr(not(feature = "coverage"), tracing::instrument(skip(state)))]
pub async fn graph_map_repo_size_fresh(
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> EndpointResult {
    let points: Vec<(String, i32, i32)> = sqlx::query_as(indoc! {r#"
        SELECT
            name,
            num_metapackages,
            num_metapackages_newest
        FROM repositories
        WHERE state != 'legacy'
    "#})
    .fetch_all(&state.pool)
    .await?;

    let repositories_data = state.repository_data_cache.snapshot();

    let points: Vec<_> = points
        .iter()
        .map(|(repository_name, x, y)| {
            if let Some(repository_data) = repositories_data.repository(repository_name) {
                Point {
                    label: repository_data.title.as_str(),
                    x: *x as u32,
                    y: *y as u32,
                    color: repository_data.brand_color.as_deref().unwrap_or("000"),
                }
            } else {
                Point {
                    label: repository_name.as_str(),
                    x: *x as u32,
                    y: *y as u32,
                    color: "000",
                }
            }
        })
        .collect();

    Ok((
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::IMAGE_SVG.as_ref()),
        )],
        render_map(
            &points,
            "Number of packages in repository",
            "Number of fresh packages in repository",
            "",
            "",
            query.xlimit,
            query.ylimit,
        ),
    )
        .into_response())
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_round_range() {
        assert_eq!(round_range(0), 1);
        assert_eq!(round_range(1), 1);

        assert_eq!(round_range(99), 99);
        assert_eq!(round_range(100), 100);
        assert_eq!(round_range(101), 110);

        assert_eq!(round_range(999), 1000);
        assert_eq!(round_range(1000), 1000);
        assert_eq!(round_range(1001), 1100);

        assert_eq!(round_range(9999), 10000);
        assert_eq!(round_range(10000), 10000);
        assert_eq!(round_range(10001), 11000);
    }
}
