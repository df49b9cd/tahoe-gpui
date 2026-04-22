//! Canvas-based mark renderers for Line, Area, Range, and Rule chart types.
//!
//! Bar and Point marks remain div-based (see `render.rs`). These functions
//! paint directly via GPUI's canvas API using `PathBuilder::stroke` / `fill`
//! + `window.paint_path`, following the pattern established in
//!
//! `examples/dashboard_app.rs`.

use std::sync::Arc;

use gpui::{Hsla, PathBuilder, Pixels, Point, Window, point, px};

use super::interpolation::{InterpolationMethod, append_interpolation};
use super::scales::Scale;
use super::stacking::StackSegment;
use super::types::{ChartPoint, ChartType, GridLineStyle, PlottableValue};

/// Dash pattern for [`GridLineStyle::Dashed`]. `(on, off)` pixels; picked
/// so a 200-pixel gridline emits ~40 dashes — dense enough to read as
/// dashed at typical chart widths without becoming dotted at small sizes.
const DASH_ON_PX: f32 = 4.0;
const DASH_OFF_PX: f32 = 3.0;

/// Paint a stroked polyline (or smooth curve when ≥3 points) connecting
/// data points projected through the supplied Y scale.
///
/// `method` selects the interpolation style used between data points —
/// see [`InterpolationMethod`] for the full vocabulary. A two-point
/// series always draws a straight segment regardless of `method`.
pub(crate) fn paint_line(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    points: &[ChartPoint],
    y_scale: &dyn Scale,
    color: Hsla,
    method: InterpolationMethod,
) {
    if points.len() < 2 {
        return;
    }
    let pts = project_points(points, origin, w, h, y_scale);
    paint_line_from_projected(window, &pts, color, method);
}

/// Project a slice of `ChartPoint`s into pixel-space coordinates using the
/// supplied scale.  Shared by `paint_line` and `paint_area` so the area
/// painter doesn't project the same points twice per frame.
fn project_points(
    points: &[ChartPoint],
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    y_scale: &dyn Scale,
) -> Vec<Point<Pixels>> {
    let denom = (points.len() as f32 - 1.0).max(1.0);
    points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let norm = y_scale.project(&p.y);
            let x = origin.x + px(w * (i as f32 / denom));
            let y = origin.y + px(h * (1.0 - norm));
            point(x, y)
        })
        .collect()
}

fn paint_line_from_projected(
    window: &mut Window,
    pts: &[Point<Pixels>],
    color: Hsla,
    method: InterpolationMethod,
) {
    if pts.len() < 2 {
        return;
    }
    let mut pb = PathBuilder::stroke(px(2.0));
    pb.move_to(pts[0]);
    append_interpolation(&mut pb, pts, method);
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

/// Paint a filled area: the same curve as `paint_line`, closed down to the
/// baseline and filled with a semi-transparent version of the stroke colour.
///
/// `method` selects the interpolation used between points for both the
/// fill envelope and the stroked upper edge. See
/// [`InterpolationMethod`] for the vocabulary.
pub(crate) fn paint_area(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    points: &[ChartPoint],
    y_scale: &dyn Scale,
    color: Hsla,
    method: InterpolationMethod,
) {
    if points.is_empty() {
        return;
    }
    let pts = project_points(points, origin, w, h, y_scale);

    let mut pb = PathBuilder::fill();

    // Start at bottom-left baseline.
    let baseline_y = origin.y + px(h);
    pb.move_to(point(origin.x, baseline_y));

    // Move to first data point, then draw smooth curve.
    pb.line_to(pts[0]);
    append_interpolation(&mut pb, &pts, method);

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

    // Stroke the upper edge for definition — reuse the projected points
    // instead of walking `paint_line` which would re-project them.
    paint_line_from_projected(window, &pts, color, method);
}

/// Paint a stacked-area ribbon bounded above by `segment.hi` and below
/// by `segment.lo` for each slot.
///
/// Stack segments come pre-normalised to `[0, 1]` (see [`StackSegment`]),
/// so this painter skips the [`Scale`] entirely and projects `1 - norm`
/// into pixel space directly. Ragged series fall through naturally:
/// slots beyond the series' own length yield empty segments whose `lo ==
/// hi`, producing a zero-height ribbon at that slot.
pub(crate) fn paint_stacked_area(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    points: &[ChartPoint],
    segments: &[StackSegment],
    color: Hsla,
    method: InterpolationMethod,
) {
    // Use the segment count as the x-axis granularity so a series shorter
    // than the slot count still emits a proportional-width ribbon.
    let n = segments.len();
    if n == 0 {
        return;
    }
    let step_x = |i: usize| origin.x + px(w * (i as f32 / (n as f32 - 1.0).max(1.0)));
    let to_y = |norm: f32| origin.y + px(h * (1.0 - norm));

    let upper: Vec<Point<Pixels>> = segments
        .iter()
        .enumerate()
        .map(|(i, s)| point(step_x(i), to_y(s.hi)))
        .collect();
    let lower: Vec<Point<Pixels>> = segments
        .iter()
        .enumerate()
        .map(|(i, s)| point(step_x(i), to_y(s.lo)))
        .collect();

    // Trace the upper edge left-to-right, then the lower edge right-to-left
    // so the band closes cleanly. The chosen interpolation smooths both
    // edges consistently so the ribbon doesn't look lopsided.
    let mut pb = PathBuilder::fill();
    pb.move_to(upper[0]);
    append_interpolation(&mut pb, &upper, method);
    let mut rev_lower = lower.clone();
    rev_lower.reverse();
    pb.line_to(rev_lower[0]);
    append_interpolation(&mut pb, &rev_lower, method);
    pb.close();

    let fill_color = Hsla {
        a: color.a * 0.45,
        ..color
    };
    if let Ok(path) = pb.build() {
        window.paint_path(path, fill_color);
    }

    // Stroke the upper edge so stacked layers remain readable even at low
    // alpha. The `points` slice is present for signature parity with
    // `paint_area`; the stacked painter itself doesn't need it for stroke
    // geometry since the edge is already projected into pixel space.
    let _ = points;
    let mut stroke = PathBuilder::stroke(px(1.0));
    stroke.move_to(upper[0]);
    append_interpolation(&mut stroke, &upper, method);
    if let Ok(path) = stroke.build() {
        window.paint_path(path, color);
    }
}

/// Paint a horizontal reference line at a single value.
pub(crate) fn paint_rule(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    value: &PlottableValue,
    y_scale: &dyn Scale,
    color: Hsla,
) {
    let norm = y_scale.project(value);
    let y = origin.y + px(h * (1.0 - norm));
    let mut pb = PathBuilder::stroke(px(1.5));
    pb.move_to(point(origin.x, y));
    pb.line_to(point(origin.x + px(w), y));
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

/// Paint a filled band between the `y` (lower) and `y_high` (upper)
/// channels of each [`ChartPoint`] (Range mark).
///
/// Points whose `y_high` is absent are silently skipped so a `Range`
/// caller passing a non-range series ends up with a zero-width band
/// rather than a panic. Truncation happens at the first such point so
/// the upper/lower edges always span the same slot count.
pub(crate) fn paint_range(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    points: &[ChartPoint],
    y_scale: &dyn Scale,
    color: Hsla,
    method: InterpolationMethod,
) {
    // Ignore non-range points — a caller passing a non-range series past
    // the first gap gets a truncated band instead of a panic.
    let count = points.iter().take_while(|p| p.y_high.is_some()).count();
    if count == 0 {
        return;
    }

    // Build low / high point sequences up front. `append_interpolation`
    // needs a slice to peek at neighbours, so we can't fuse this into
    // a single reversed iterator.
    let high_pts: Vec<Point<Pixels>> = points
        .iter()
        .take(count)
        .enumerate()
        .map(|(i, p)| {
            let norm = p.y_high.as_ref().map(|v| y_scale.project(v)).unwrap_or(0.0);
            let x = origin.x + px(w * (i as f32 / (count as f32 - 1.0).max(1.0)));
            let y = origin.y + px(h * (1.0 - norm));
            point(x, y)
        })
        .collect();
    let mut low_pts: Vec<Point<Pixels>> = points
        .iter()
        .take(count)
        .enumerate()
        .map(|(i, p)| {
            let norm = y_scale.project(&p.y);
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
    append_interpolation(&mut pb, &high_pts, method);

    // Lower edge, right to left — reverse in place so we don't allocate a
    // second Vec just to walk backwards through the same data.
    low_pts.reverse();
    pb.line_to(low_pts[0]);
    append_interpolation(&mut pb, &low_pts, method);

    pb.close();

    if let Ok(path) = pb.build() {
        window.paint_path(path, fill_color);
    }

    // Stroke upper and lower edges for definition. We synthesise
    // single-channel point sequences so `paint_line` reads y from each
    // point; reusing the same `points` slice would stroke along `y`
    // (lower) even when we wanted the `y_high` edge.
    let high_edge: Vec<ChartPoint> = points
        .iter()
        .take(count)
        .map(|p| {
            ChartPoint::new(
                p.x.clone(),
                p.y_high.clone().unwrap_or(PlottableValue::Number(0.0)),
            )
        })
        .collect();
    let low_edge: Vec<ChartPoint> = points
        .iter()
        .take(count)
        .map(|p| ChartPoint::new(p.x.clone(), p.y.clone()))
        .collect();
    paint_line(window, origin, w, h, &high_edge, y_scale, color, method);
    paint_line(window, origin, w, h, &low_edge, y_scale, color, method);
}

/// The canvas callback type expected by the render path.
pub(crate) type PaintFn = Box<dyn FnOnce(&mut Window)>;

/// Build the canvas paint callback for the given chart type.
///
/// Returns `None` for Bar and Point (those use div-based rendering).
/// For Range charts, each [`ChartPoint`] carries its own upper bound via
/// `y_high`.
///
/// Takes `points` as `Arc<[ChartPoint]>` and `y_scale` as `Arc<dyn Scale>`
/// so the closure captures refcount clones of both instead of deep copies.
pub(crate) fn canvas_paint_callback(
    chart_type: ChartType,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    points: Arc<[ChartPoint]>,
    y_scale: Arc<dyn Scale>,
    color: Hsla,
    method: InterpolationMethod,
) -> Option<PaintFn> {
    match chart_type {
        ChartType::Line => Some(Box::new(move |window: &mut Window| {
            paint_line(window, origin, w, h, &points, &*y_scale, color, method);
        })),
        ChartType::Area => Some(Box::new(move |window: &mut Window| {
            paint_area(window, origin, w, h, &points, &*y_scale, color, method);
        })),
        ChartType::Rule => {
            debug_assert!(
                points.len() <= 1,
                "ChartType::Rule draws only the first point's y — extra points are ignored"
            );
            // Rule with no values is nothing to paint — returning a closure
            // that drew `0.0` added a phantom reference line at the chart's
            // zero baseline, which a caller inspecting the empty case would
            // mistake for data.
            let value = points.first().map(|p| p.y.clone())?;
            Some(Box::new(move |window: &mut Window| {
                paint_rule(window, origin, w, h, &value, &*y_scale, color);
            }))
        }
        ChartType::Range => Some(Box::new(move |window: &mut Window| {
            paint_range(window, origin, w, h, &points, &*y_scale, color, method);
        })),
        ChartType::Bar | ChartType::Point | ChartType::Sector | ChartType::Rectangle => None,
    }
}

/// Paint horizontal gridlines at Y-axis tick positions.
///
/// All ticks are batched into a single `PathBuilder` (one `move_to` +
/// `line_to` per tick, or a run of dash segments when `style` is
/// [`GridLineStyle::Dashed`]) and painted in a single `paint_path`
/// call. A 5-tick solid axis costs one path; a dashed axis costs one
/// path with `5 × (dash_count)` segments.
///
/// `style == GridLineStyle::Hidden` short-circuits to a no-op.
pub(crate) fn paint_horizontal_gridlines(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    ticks: &[PlottableValue],
    y_scale: &dyn Scale,
    color: Hsla,
    style: GridLineStyle,
) {
    if ticks.is_empty() || matches!(style, GridLineStyle::Hidden) {
        return;
    }
    let mut pb = PathBuilder::stroke(px(0.5));
    for tick in ticks {
        let norm = y_scale.project(tick);
        let y = origin.y + px(h * (1.0 - norm));
        append_horizontal_stroke(&mut pb, origin.x, y, w, style);
    }
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

/// Paint vertical gridlines at each data point slot.
///
/// Batches all subpaths into a single `PathBuilder` for a single
/// `paint_path` call. A dashed style emits a run of short segments per
/// gridline — see [`paint_horizontal_gridlines`] for the shared pattern.
pub(crate) fn paint_vertical_gridlines(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    count: usize,
    color: Hsla,
    style: GridLineStyle,
) {
    if count <= 1 || matches!(style, GridLineStyle::Hidden) {
        return;
    }
    let slot_width = w / count as f32;
    let mut pb = PathBuilder::stroke(px(0.5));
    for i in 1..count {
        let x = origin.x + px(slot_width * i as f32);
        append_vertical_stroke(&mut pb, x, origin.y, h, style);
    }
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

/// Emit a single horizontal line or dash run into `pb`.
fn append_horizontal_stroke(
    pb: &mut PathBuilder,
    start_x: Pixels,
    y: Pixels,
    w: f32,
    style: GridLineStyle,
) {
    match style {
        GridLineStyle::Solid | GridLineStyle::Hidden => {
            pb.move_to(point(start_x, y));
            pb.line_to(point(start_x + px(w), y));
        }
        GridLineStyle::Dashed => {
            let mut cursor = 0.0_f32;
            while cursor < w {
                let on_end = (cursor + DASH_ON_PX).min(w);
                pb.move_to(point(start_x + px(cursor), y));
                pb.line_to(point(start_x + px(on_end), y));
                cursor = on_end + DASH_OFF_PX;
            }
        }
    }
}

/// Emit a single vertical line or dash run into `pb`.
fn append_vertical_stroke(
    pb: &mut PathBuilder,
    x: Pixels,
    start_y: Pixels,
    h: f32,
    style: GridLineStyle,
) {
    match style {
        GridLineStyle::Solid | GridLineStyle::Hidden => {
            pb.move_to(point(x, start_y));
            pb.line_to(point(x, start_y + px(h)));
        }
        GridLineStyle::Dashed => {
            let mut cursor = 0.0_f32;
            while cursor < h {
                let on_end = (cursor + DASH_ON_PX).min(h);
                pb.move_to(point(x, start_y + px(cursor)));
                pb.line_to(point(x, start_y + px(on_end)));
                cursor = on_end + DASH_OFF_PX;
            }
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
