// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use itertools::Itertools;

use crate::xmlwriter::{xml, XmlTag};

#[derive(Clone, Copy)]
pub enum GraphType {
    Integer,
    Float,
}

impl GraphType {
    fn steps(&self) -> &[f32] {
        match self {
            GraphType::Integer => {
                const STEPS: &[f32] = &[
                    1., 2., 5., 10., 20., 50., 100., 200., 500., 1000., 2000., 5000., 10000.,
                    20000., 50000., 100000.,
                ];
                STEPS
            }
            GraphType::Float => {
                const STEPS: &[f32] = &[
                    0.0001, 0.0002, 0.0005, 0.001, 0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5,
                    1., 2., 5., 10., 20., 50., 100., 200., 500., 1000., 2000., 5000., 10000.,
                    20000., 50000., 100000.,
                ];
                STEPS
            }
        }
    }
}

#[derive(Default, Debug)]
struct NormalizedGraphData {
    points: Vec<(f32, f32)>,
    y_ticks: Vec<(f32, String)>,
}

#[derive(Debug)]
struct Ticks {
    start: f32,
    step: f32,
    count: usize,
    precision: usize,
}

fn calculate_ticks(min: f32, max: f32, graph_type: GraphType) -> Ticks {
    assert!(max > min);

    let mut step = (max - min) / 10.;

    for &round_step in graph_type.steps() {
        if round_step >= step {
            step = round_step;
            break;
        }
    }

    let precision = if step < 1. {
        (-step.log10().floor()) as usize
    } else {
        0
    };

    let start = (min / step).ceil() * step;

    Ticks {
        start,
        step,
        count: ((max - start) / step).floor() as usize + 1,
        precision,
    }
}

fn collect_with_deduplication_by_value<I, A, B>(iter: I) -> Vec<(A, B)>
where
    A: Copy,
    B: Copy + PartialEq,
    I: Iterator<Item = (A, B)>,
{
    let mut res = Vec::<(A, B)>::new();
    for item in iter {
        let len = res.len();
        if len >= 2 && res[len - 1].1 == item.1 && res[len - 2].1 == item.1 {
            res.pop();
        }
        res.push(item);
    }
    res
}

fn normalize_graph_data(
    points: &Vec<(Duration, f32)>,
    graph_type: GraphType,
    period: Duration,
    suffix: &str,
) -> NormalizedGraphData {
    match points.iter().map(|(_, value)| value).minmax().into_option() {
        None => Default::default(),
        Some((&min, &max)) if min == max => NormalizedGraphData {
            points: vec![
                (
                    points.first().unwrap().0.as_secs_f32() / period.as_secs_f32(),
                    0.5,
                ),
                (
                    points.last().unwrap().0.as_secs_f32() / period.as_secs_f32(),
                    0.5,
                ),
            ],
            y_ticks: vec![(0.5, format!("{}{}", min, suffix))],
        },
        Some((&min, &max)) => {
            let ticks = calculate_ticks(min, max, graph_type);
            NormalizedGraphData {
                points: collect_with_deduplication_by_value(points.iter().map(|(age, value)| {
                    (
                        age.as_secs_f32() / period.as_secs_f32(),
                        (value - min) / (max - min),
                    )
                })),
                y_ticks: (0..ticks.count)
                    .map(|i| {
                        let value = ticks.start + ticks.step * (i as f32);

                        (
                            (value - min) / (max - min),
                            format!(
                                "{:.precision$}{}",
                                value,
                                suffix,
                                precision = ticks.precision
                            ),
                        )
                    })
                    .collect(),
            }
        }
    }
}

pub fn render_graph(
    points: &Vec<(Duration, f32)>,
    graph_type: GraphType,
    period: Duration,
    suffix: &str,
    stroke: &str,
) -> String {
    let period_days = period.as_secs() / (60 * 60 * 24);

    const IMAGE_WIDTH: u64 = 1140;
    const IMAGE_HEIGHT: u64 = 400;
    const GRAPH_AREA_MARGIN_RIGHT: u64 = 50;
    const GRAPH_AREA_MARGIN_TOP: u64 = 10;
    const GRAPH_AREA_MARGIN_BOTTOM: u64 = 30;
    const TEXT_HEIGHT: u64 = 20;
    const GRAPH_AREA_WIDTH: u64 = IMAGE_WIDTH - GRAPH_AREA_MARGIN_RIGHT;
    const GRAPH_AREA_HEIGHT: u64 = IMAGE_HEIGHT - GRAPH_AREA_MARGIN_BOTTOM - GRAPH_AREA_MARGIN_TOP;

    let data = normalize_graph_data(points, graph_type, Duration::from_days(period_days), suffix);

    let pixel_x = |x: f32| ((1.0 - x) * (GRAPH_AREA_WIDTH as f32)).floor() + 0.5;
    let pixel_y = |y: f32| {
        (GRAPH_AREA_MARGIN_TOP as f32 + (1.0 - y) * (GRAPH_AREA_HEIGHT as f32)).floor() + 0.5
    };

    // start SVG document
    let mut doc = xml!(
        "svg",
        "xmlns" = "http://www.w3.org/2000/svg",
        "width" = IMAGE_WIDTH,
        "height" = IMAGE_HEIGHT
    );

    let mut x_tick_texts = xml!(
        "g",
        "fill" = "#000",
        "text-anchor" = "middle",
        "font-family" = "DejaVu Sans,Verdana,Geneva,sans-serif",
        "font-size" = 11
    );
    let mut y_tick_texts = xml!(
        "g",
        "fill" = "#000",
        "text-anchor" = "left",
        "font-family" = "DejaVu Sans,Verdana,Geneva,sans-serif",
        "font-size" = 11,
        "alignment-baseline" = "middle"
    );

    // day grid
    for day in 0..period_days {
        let x = pixel_x((day + 1) as f32 / period_days as f32);
        let width = GRAPH_AREA_WIDTH as f32 / period_days as f32;

        if day % 2 == 0 {
            doc.add_child(xml!(
                "rect",
                "x" = x,
                "width" = width,
                "height" = IMAGE_HEIGHT,
                "fill" = "#f0f0f0"
            ));
        }

        if day != period_days - 1 {
            x_tick_texts.add_child(
                xml!("text", "x" = x, "y" = IMAGE_HEIGHT - 5).with_text(&format!("-{}d", day + 1)),
            );
        }
    }

    // vertical grid lines
    for line in 0..(period_days * 4 - 1) {
        let (height, stroke) = if line % 4 == 3 {
            (IMAGE_HEIGHT - TEXT_HEIGHT, "#c0c0c0")
        } else {
            (IMAGE_HEIGHT, "#e0e0e0")
        };

        let x = pixel_x((line + 1) as f32 / period_days as f32 / 4.0);
        doc.add_child(xml!(
            "line",
            "x1" = x,
            "x2" = x,
            "y1" = 0,
            "y2" = height,
            "stroke" = stroke
        ));
    }

    // horizontal grid lines
    for tick in data.y_ticks {
        let y = pixel_y(tick.0);
        doc.add_child(xml!(
            "line",
            "x1" = 0,
            "x2" = GRAPH_AREA_WIDTH,
            "y1" = y,
            "y2" = y,
            "stroke" = "#c0c0c0"
        ));
        y_tick_texts.add_child(
            xml!(
                "text",
                "x" = GRAPH_AREA_WIDTH + 2,
                "y" = y + 3.0,
                "alignment-baseline" = "middle"
            )
            .with_text(&tick.1),
        );
    }

    // graph data
    let mut graph_lines = xml!(
        "g",
        "stroke" = stroke,
        "stroke-width" = 3,
        "stroke-linecap" = "round"
    );

    for (a, b) in data.points.iter().tuple_windows() {
        graph_lines.add_child(xml!(
            "line",
            "x1" = pixel_x(b.0),
            "y1" = pixel_y(b.1),
            "x2" = pixel_x(a.0),
            "y2" = pixel_y(a.1)
        ));
    }

    doc.add_child(graph_lines);
    doc.add_child(x_tick_texts);
    doc.add_child(y_tick_texts);

    doc.to_string()
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    use float_cmp::approx_eq;

    #[test]
    fn test_collect_with_deduplication_by_value() {
        let inp = vec![(1, 1), (2, 1), (3, 2), (4, 2), (5, 3), (6, 3)];
        assert_eq!(
            collect_with_deduplication_by_value(inp.clone().into_iter()),
            inp
        );

        let inp = vec![
            (1, 1),
            (2, 1),
            (3, 1),
            (4, 2),
            (5, 2),
            (6, 2),
            (7, 3),
            (8, 3),
            (9, 3),
        ];
        let out = vec![(1, 1), (3, 1), (4, 2), (6, 2), (7, 3), (9, 3)];
        assert_eq!(collect_with_deduplication_by_value(inp.into_iter()), out);
    }

    impl PartialEq for NormalizedGraphData {
        fn eq(&self, other: &Self) -> bool {
            self.points
                .iter()
                .zip(other.points.iter())
                .all(|(a, b)| approx_eq!(f32, a.0, b.0) && approx_eq!(f32, a.1, b.1))
                && self
                    .y_ticks
                    .iter()
                    .zip(other.y_ticks.iter())
                    .all(|(a, b)| approx_eq!(f32, a.0, b.0) && a.1 == b.1)
        }
    }

    #[test]
    fn test_normalize_graph_data_empty() {
        assert_eq!(
            normalize_graph_data(&vec![], GraphType::Integer, Duration::from_days(1), "pcs"),
            NormalizedGraphData {
                points: vec![],
                y_ticks: vec![]
            }
        );
    }

    #[test]
    fn test_normalize_graph_data_one() {
        assert_eq!(
            normalize_graph_data(
                &vec![(Duration::from_hours(12), 1.0)],
                GraphType::Integer,
                Duration::from_days(1),
                "pcs"
            ),
            NormalizedGraphData {
                points: vec![(0.5, 0.5), (0.5, 0.5)],
                y_ticks: vec![(0.5, String::from("1pcs"))],
            }
        );
    }

    #[test]
    fn test_normalize_graph_data_flat() {
        assert_eq!(
            normalize_graph_data(
                &vec![
                    (Duration::from_hours(18), 3.0),
                    (Duration::from_hours(6), 3.0)
                ],
                GraphType::Integer,
                Duration::from_days(1),
                "pcs"
            ),
            NormalizedGraphData {
                points: vec![(0.75, 0.5), (0.25, 0.5)],
                y_ticks: vec![(0.5, String::from("3pcs"))],
            }
        );
    }

    #[test]
    fn test_normalize_graph_data_generic_base() {
        assert_eq!(
            normalize_graph_data(
                &vec![
                    (Duration::from_hours(24), 0.0),
                    (Duration::from_hours(12), 2.5),
                    (Duration::from_hours(0), 10.0)
                ],
                GraphType::Integer,
                Duration::from_days(1),
                "pcs"
            ),
            NormalizedGraphData {
                points: vec![(1.0, 0.0), (0.5, 0.25), (0.0, 1.0)],
                y_ticks: vec![
                    (0.0, String::from("0pcs")),
                    (0.1, String::from("1pcs")),
                    (0.2, String::from("2pcs")),
                    (0.3, String::from("3pcs")),
                    (0.4, String::from("4pcs")),
                    (0.5, String::from("5pcs")),
                    (0.6, String::from("6pcs")),
                    (0.7, String::from("7pcs")),
                    (0.8, String::from("8pcs")),
                    (0.9, String::from("9pcs")),
                    (1.0, String::from("10pcs")),
                ],
            }
        );
    }

    #[test]
    fn test_normalize_graph_data_generic_ticks_1() {
        // still 10 ticks, and tick period is round
        assert_eq!(
            normalize_graph_data(
                &vec![
                    (Duration::from_hours(24), 0.0),
                    (Duration::from_hours(12), 2.25),
                    (Duration::from_hours(0), 9.0)
                ],
                GraphType::Integer,
                Duration::from_days(1),
                "pcs"
            ),
            NormalizedGraphData {
                points: vec![(1.0, 0.0), (0.5, 0.25), (0.0, 1.0)],
                y_ticks: vec![
                    (0.0, String::from("0pcs")),
                    (1. / 9., String::from("1pcs")),
                    (2. / 9., String::from("2pcs")),
                    (3. / 9., String::from("3pcs")),
                    (4. / 9., String::from("4pcs")),
                    (5. / 9., String::from("5pcs")),
                    (6. / 9., String::from("6pcs")),
                    (7. / 9., String::from("7pcs")),
                    (8. / 9., String::from("8pcs")),
                    (1.0, String::from("9pcs")),
                ],
            }
        );
    }

    #[test]
    fn test_normalize_graph_data_generic_ticks_2() {
        // 11 ticks is too much so it doubles the interval and switches to 6 tocks
        assert_eq!(
            normalize_graph_data(
                &vec![
                    (Duration::from_hours(24), 0.0),
                    (Duration::from_hours(12), 2.75),
                    (Duration::from_hours(0), 11.0)
                ],
                GraphType::Integer,
                Duration::from_days(1),
                "pcs"
            ),
            NormalizedGraphData {
                points: vec![(1.0, 0.0), (0.5, 0.25), (0.0, 1.0)],
                y_ticks: vec![
                    (0.0, String::from("0pcs")),
                    (2. / 11., String::from("2pcs")),
                    (4. / 11., String::from("4pcs")),
                    (6. / 11., String::from("6pcs")),
                    (8. / 11., String::from("8pcs")),
                    (10. / 11., String::from("10pcs")),
                ],
            }
        );
    }

    #[test]
    fn test_normalize_graph_data_ticks_incorrect_calculation() {
        // regression where tick count was calculated incorrectly
        // and there was an extra tick outside the graph
        let data = normalize_graph_data(
            &vec![
                (Duration::from_hours(24), 6514.0),
                (Duration::from_hours(0), 6745.0),
            ],
            GraphType::Integer,
            Duration::from_days(1),
            "",
        );
        assert_eq!(
            data.y_ticks
                .iter()
                .map(|(_, label)| label.as_str())
                .collect::<Vec<_>>(),
            vec!["6550", "6600", "6650", "6700"]
        );
    }

    #[test]
    fn test_normalize_graph_data_generic_base_float() {
        // still whole ticks
        assert_eq!(
            normalize_graph_data(
                &vec![
                    (Duration::from_hours(24), 0.0),
                    (Duration::from_hours(12), 2.5),
                    (Duration::from_hours(0), 10.0)
                ],
                GraphType::Float,
                Duration::from_days(1),
                "pcs"
            ),
            NormalizedGraphData {
                points: vec![(1.0, 0.0), (0.5, 0.25), (0.0, 1.0)],
                y_ticks: vec![
                    (0.0, String::from("0pcs")),
                    (0.1, String::from("1pcs")),
                    (0.2, String::from("2pcs")),
                    (0.3, String::from("3pcs")),
                    (0.4, String::from("4pcs")),
                    (0.5, String::from("5pcs")),
                    (0.6, String::from("6pcs")),
                    (0.7, String::from("7pcs")),
                    (0.8, String::from("8pcs")),
                    (0.9, String::from("9pcs")),
                    (1.0, String::from("10pcs")),
                ],
            }
        );
    }

    #[test]
    fn test_normalize_graph_data_generic_base_float_1() {
        // switches to fractional ticks
        assert_eq!(
            normalize_graph_data(
                &vec![
                    (Duration::from_hours(24), 0.0),
                    (Duration::from_hours(12), 0.5),
                    (Duration::from_hours(0), 2.0)
                ],
                GraphType::Float,
                Duration::from_days(1),
                "pcs"
            ),
            NormalizedGraphData {
                points: vec![(1.0, 0.0), (0.5, 0.25), (0.0, 1.0)],
                y_ticks: vec![
                    (0.0, String::from("0.0pcs")),
                    (0.1, String::from("0.2pcs")),
                    (0.2, String::from("0.4pcs")),
                    (0.3, String::from("0.6pcs")),
                    (0.4, String::from("0.8pcs")),
                    (0.5, String::from("1.0pcs")),
                    (0.6, String::from("1.2pcs")),
                    (0.7, String::from("1.4pcs")),
                    (0.8, String::from("1.6pcs")),
                    (0.9, String::from("1.8pcs")),
                    (1.0, String::from("2.0pcs")),
                ],
            }
        );
    }
}
