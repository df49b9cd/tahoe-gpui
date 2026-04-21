//! Canvas-based mark renderers for Line, Area, Range, and Rule chart types.
//!
//! Bar and Point marks remain div-based (see `render.rs`). These functions
//! paint directly via GPUI's canvas API using `PathBuilder::stroke` / `fill`
//! + `window.paint_path`, following the pattern established in
//!
//! `examples/dashboard_app.rs`.

use gpui::{Hsla, PathBuilder, Pixels, Point, Window, point, px};

use super::types::ChartType;

/// Paint a stroked polyline (or smooth curve when ≥3 points) connecting
/// normalised data points.
pub(crate) fn paint_line(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    values: &[f32],
    min: f32,
    range: f32,
    color: Hsla,
) {
    if values.len() < 2 {
        return;
    }
    let pts: Vec<Point<Pixels>> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let norm = ((v - min) / range).clamp(0.0, 1.0);
            let x = origin.x + px(w * (i as f32 / (values.len() as f32 - 1.0)));
            let y = origin.y + px(h * (1.0 - norm));
            point(x, y)
        })
        .collect();

    let mut pb = PathBuilder::stroke(px(2.0));
    pb.move_to(pts[0]);
    if pts.len() >= 3 {
        append_catmull_rom(&mut pb, &pts);
    } else {
        pb.line_to(pts[1]);
    }
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

/// Append Catmull-Rom spline segments (converted to cubic beziers) to the
/// path builder. Produces a smooth curve that passes through every point.
fn append_catmull_rom(pb: &mut gpui::PathBuilder, pts: &[Point<Pixels>]) {
    let n = pts.len();
    if n < 2 {
        return;
    }
    let tension: f32 = 1.0;

    for i in 0..n - 1 {
        let p0 = if i == 0 { pts[0] } else { pts[i - 1] };
        let p1 = pts[i];
        let p2 = pts[i + 1];
        let p3 = if i + 2 < n { pts[i + 2] } else { pts[n - 1] };

        let cp1 = point(
            p1.x + px((f32::from(p2.x) - f32::from(p0.x)) / (6.0 * tension)),
            p1.y + px((f32::from(p2.y) - f32::from(p0.y)) / (6.0 * tension)),
        );
        let cp2 = point(
            p2.x - px((f32::from(p3.x) - f32::from(p1.x)) / (6.0 * tension)),
            p2.y - px((f32::from(p3.y) - f32::from(p1.y)) / (6.0 * tension)),
        );

        pb.cubic_bezier_to(p2, cp1, cp2);
    }
}

/// Paint a filled area: the same curve as `paint_line`, closed down to the
/// baseline and filled with a semi-transparent version of the stroke colour.
pub(crate) fn paint_area(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    values: &[f32],
    min: f32,
    range: f32,
    color: Hsla,
) {
    if values.is_empty() {
        return;
    }
    let pts: Vec<Point<Pixels>> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let norm = ((v - min) / range).clamp(0.0, 1.0);
            let x = origin.x + px(w * (i as f32 / (values.len() as f32 - 1.0).max(1.0)));
            let y = origin.y + px(h * (1.0 - norm));
            point(x, y)
        })
        .collect();

    let mut pb = PathBuilder::fill();

    // Start at bottom-left baseline.
    let baseline_y = origin.y + px(h);
    pb.move_to(point(origin.x, baseline_y));

    // Move to first data point, then draw smooth curve.
    pb.line_to(pts[0]);
    if pts.len() >= 3 {
        append_catmull_rom(&mut pb, &pts);
    } else if pts.len() == 2 {
        pb.line_to(pts[1]);
    }

    // Close back to baseline.
    let last_x = origin.x + px(w);
    pb.line_to(point(last_x, baseline_y));
    pb.close();

    let fill_color = Hsla {
        a: color.a * 0.35,
        ..color
    };
    if let Ok(path) = pb.build() {
        window.paint_path(path, fill_color);
    }

    // Stroke the upper edge for definition.
    paint_line(window, origin, w, h, values, min, range, color);
}

/// Paint a horizontal reference line at a single value.
pub(crate) fn paint_rule(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    value: f32,
    min: f32,
    range: f32,
    color: Hsla,
) {
    let norm = ((value - min) / range).clamp(0.0, 1.0);
    let y = origin.y + px(h * (1.0 - norm));
    let mut pb = PathBuilder::stroke(px(1.5));
    pb.move_to(point(origin.x, y));
    pb.line_to(point(origin.x + px(w), y));
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

/// Paint a filled band between lower and upper value arrays (Range mark).
pub(crate) fn paint_range(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    values_low: &[f32],
    values_high: &[f32],
    min: f32,
    range: f32,
    color: Hsla,
) {
    let count = values_low.len().min(values_high.len());
    if count == 0 {
        return;
    }

    let high_pts: Vec<Point<Pixels>> = values_high
        .iter()
        .take(count)
        .enumerate()
        .map(|(i, v)| {
            let norm = ((v - min) / range).clamp(0.0, 1.0);
            let x = origin.x + px(w * (i as f32 / (count as f32 - 1.0).max(1.0)));
            let y = origin.y + px(h * (1.0 - norm));
            point(x, y)
        })
        .collect();
    let low_pts: Vec<Point<Pixels>> = values_low
        .iter()
        .take(count)
        .enumerate()
        .map(|(i, v)| {
            let norm = ((v - min) / range).clamp(0.0, 1.0);
            let x = origin.x + px(w * (i as f32 / (count as f32 - 1.0).max(1.0)));
            let y = origin.y + px(h * (1.0 - norm));
            point(x, y)
        })
        .collect();

    let fill_color = Hsla {
        a: color.a * 0.4,
        ..color
    };
    let mut pb = PathBuilder::fill();

    // Upper edge, left to right.
    pb.move_to(high_pts[0]);
    if high_pts.len() >= 3 {
        append_catmull_rom(&mut pb, &high_pts);
    } else if high_pts.len() == 2 {
        pb.line_to(high_pts[1]);
    }

    // Lower edge, right to left.
    let rev_low: Vec<Point<Pixels>> = low_pts.iter().copied().rev().collect();
    pb.line_to(rev_low[0]);
    if rev_low.len() >= 3 {
        append_catmull_rom(&mut pb, &rev_low);
    } else if rev_low.len() == 2 {
        pb.line_to(rev_low[1]);
    }

    pb.close();

    if let Ok(path) = pb.build() {
        window.paint_path(path, fill_color);
    }

    // Stroke upper and lower edges for definition.
    paint_line(window, origin, w, h, values_high, min, range, color);
    paint_line(window, origin, w, h, values_low, min, range, color);
}

/// The canvas callback type expected by the render path.
pub(crate) type PaintFn = Box<dyn FnOnce(&mut Window)>;

/// Build the canvas paint callback for the given chart type.
///
/// Returns `None` for Bar and Point (those use div-based rendering).
/// For Range charts, `range_low` provides the lower-bound values.
pub(crate) fn canvas_paint_callback(
    chart_type: ChartType,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    values: &[f32],
    range_low: Option<&[f32]>,
    min: f32,
    range: f32,
    color: Hsla,
) -> Option<PaintFn> {
    match chart_type {
        ChartType::Line => {
            let values = values.to_vec();
            Some(Box::new(move |window: &mut Window| {
                paint_line(window, origin, w, h, &values, min, range, color);
            }))
        }
        ChartType::Area => {
            let values = values.to_vec();
            Some(Box::new(move |window: &mut Window| {
                paint_area(window, origin, w, h, &values, min, range, color);
            }))
        }
        ChartType::Rule => {
            let value = values.first().copied().unwrap_or(0.0);
            Some(Box::new(move |window: &mut Window| {
                paint_rule(window, origin, w, h, value, min, range, color);
            }))
        }
        ChartType::Range => {
            let high = values.to_vec();
            let low = range_low
                .map(|l| l.to_vec())
                .unwrap_or_else(|| high.clone());
            Some(Box::new(move |window: &mut Window| {
                paint_range(window, origin, w, h, &low, &high, min, range, color);
            }))
        }
        ChartType::Bar | ChartType::Point => None,
    }
}

/// Paint horizontal gridlines at Y-axis tick positions.
pub(crate) fn paint_horizontal_gridlines(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    ticks: &[f32],
    min: f32,
    range: f32,
    color: Hsla,
) {
    for tick in ticks {
        let norm = ((tick - min) / range).clamp(0.0, 1.0);
        let y = origin.y + px(h * (1.0 - norm));
        let mut pb = PathBuilder::stroke(px(0.5));
        pb.move_to(point(origin.x, y));
        pb.line_to(point(origin.x + px(w), y));
        if let Ok(path) = pb.build() {
            window.paint_path(path, color);
        }
    }
}

/// Paint vertical gridlines at each data point slot.
pub(crate) fn paint_vertical_gridlines(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    count: usize,
    color: Hsla,
) {
    if count <= 1 {
        return;
    }
    let slot_width = w / count as f32;
    for i in 1..count {
        let x = origin.x + px(slot_width * i as f32);
        let mut pb = PathBuilder::stroke(px(0.5));
        pb.move_to(point(x, origin.y));
        pb.line_to(point(x, origin.y + px(h)));
        if let Ok(path) = pb.build() {
            window.paint_path(path, color);
        }
    }
}

/// Paint the Y-axis line at the left edge of the plot area.
pub(crate) fn paint_y_axis_line(window: &mut Window, origin: Point<Pixels>, h: f32, color: Hsla) {
    let mut pb = PathBuilder::stroke(px(1.0));
    pb.move_to(point(origin.x, origin.y));
    pb.line_to(point(origin.x, origin.y + px(h)));
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}
