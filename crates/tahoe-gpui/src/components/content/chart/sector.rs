//! Sector (pie / donut) mark renderer.
//!
//! Each series contributes one slice sized by the first point's numeric
//! `y` value. Slices are arranged around a common centre, starting at
//! `start_angle` and sweeping clockwise. Inner radius > 0 produces a
//! donut; inner radius == 0 produces a solid pie.
//!
//! Paint strategy mirrors `marks.rs`: build paths via `PathBuilder::fill`
//! and `arc_to`, then pass them to `window.paint_path`. Overlapping
//! stripes (when Differentiate Without Color is active) ride on top as
//! a second stroke pass using the same sector boundary.
//!
//! # HIG
//!
//! <https://developer.apple.com/design/human-interface-guidelines/charts>

use std::f32::consts::{PI, TAU};

use gpui::{Hsla, PathBuilder, Pixels, Point, Window, point, px};

use super::types::ChartDataSet;

/// Default start angle for sector layouts — 12 o'clock (top of the ring).
pub(crate) const DEFAULT_SECTOR_START_ANGLE: f32 = -PI / 2.0;

/// Render a full sector chart into a canvas callback.
///
/// The origin + size describe the bounding box; the ring is inscribed into
/// a centred square inside that box so the pie stays round even when the
/// plot area is wider than it is tall. Series whose first-point `y` is
/// non-numeric or non-positive are skipped.
pub(crate) fn paint_sector_chart(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    weights: &[(Hsla, f32)],
    inner_radius_ratio: f32,
    start_angle: f32,
    dwc: bool,
) {
    let total: f32 = weights.iter().map(|(_, v)| *v).sum();
    if !total.is_finite() || total <= 0.0 {
        return;
    }

    // Inscribe the ring in a centred square so the pie stays round.
    let diameter = w.min(h);
    let radius = diameter * 0.5;
    let cx = origin.x + px(w * 0.5);
    let cy = origin.y + px(h * 0.5);

    let inner_radius = radius * inner_radius_ratio.clamp(0.0, 0.95);

    let mut cursor = start_angle;
    for (i, (color, value)) in weights.iter().enumerate() {
        if *value <= 0.0 || !value.is_finite() {
            continue;
        }
        let sweep = (value / total) * TAU;
        let end_angle = cursor + sweep;

        paint_slice(
            window,
            cx,
            cy,
            radius,
            inner_radius,
            cursor,
            end_angle,
            *color,
        );
        if dwc {
            paint_slice_texture(window, cx, cy, radius, inner_radius, cursor, end_angle, i);
        }

        cursor = end_angle;
    }
}

/// Paint a single slice (solid fill) between `start` and `end` angles.
fn paint_slice(
    window: &mut Window,
    cx: Pixels,
    cy: Pixels,
    outer: f32,
    inner: f32,
    start: f32,
    end: f32,
    color: Hsla,
) {
    let sweep = end - start;
    // lyon's `arc_to` can't represent a full 360° sweep with a single
    // call; when a single slice covers the whole ring, split it at the
    // halfway angle so both arcs stay <= 180°.
    let full_circle = (sweep - TAU).abs() < 1e-4;
    let large_arc = sweep.abs() > PI;

    let (sx, sy) = polar(cx, cy, outer, start);
    let (ex, ey) = polar(cx, cy, outer, end);

    let mut pb = PathBuilder::fill();

    if inner <= 0.5 {
        // Pie slice — wedge anchored at centre.
        pb.move_to(point(cx, cy));
        pb.line_to(point(sx, sy));
        if full_circle {
            let (mx, my) = polar(cx, cy, outer, start + sweep * 0.5);
            pb.arc_to(
                point(px(outer), px(outer)),
                px(0.0),
                false,
                true,
                point(mx, my),
            );
            pb.arc_to(
                point(px(outer), px(outer)),
                px(0.0),
                false,
                true,
                point(ex, ey),
            );
        } else {
            pb.arc_to(
                point(px(outer), px(outer)),
                px(0.0),
                large_arc,
                true,
                point(ex, ey),
            );
        }
        pb.close();
    } else {
        // Donut slice — trapezoid-like ring segment.
        let (isx, isy) = polar(cx, cy, inner, start);
        let (iex, iey) = polar(cx, cy, inner, end);

        pb.move_to(point(sx, sy));
        if full_circle {
            let (mx, my) = polar(cx, cy, outer, start + sweep * 0.5);
            pb.arc_to(
                point(px(outer), px(outer)),
                px(0.0),
                false,
                true,
                point(mx, my),
            );
            pb.arc_to(
                point(px(outer), px(outer)),
                px(0.0),
                false,
                true,
                point(ex, ey),
            );
        } else {
            pb.arc_to(
                point(px(outer), px(outer)),
                px(0.0),
                large_arc,
                true,
                point(ex, ey),
            );
        }
        pb.line_to(point(iex, iey));
        if full_circle {
            let (imx, imy) = polar(cx, cy, inner, start + sweep * 0.5);
            pb.arc_to(
                point(px(inner), px(inner)),
                px(0.0),
                false,
                false,
                point(imx, imy),
            );
            pb.arc_to(
                point(px(inner), px(inner)),
                px(0.0),
                false,
                false,
                point(isx, isy),
            );
        } else {
            pb.arc_to(
                point(px(inner), px(inner)),
                px(0.0),
                large_arc,
                false,
                point(isx, isy),
            );
        }
        pb.close();
    }

    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

/// Paint a Differentiate-Without-Color texture pass on top of a slice.
///
/// GPUI has no hatch primitive so each texture is emulated by painting
/// parallel 1-px strokes inside the slice's bounding arc. Three textures
/// are cycled so adjacent slices differ.
fn paint_slice_texture(
    window: &mut Window,
    cx: Pixels,
    cy: Pixels,
    outer: f32,
    inner: f32,
    start: f32,
    end: f32,
    index: usize,
) {
    // Rotate direction by slice index so neighbours diverge visibly.
    let step = match index % 3 {
        0 => return, // solid — no overlay pass.
        1 => 10.0,   // dense dots emulated as short tick marks.
        _ => 6.0,    // stripes.
    };

    let accent = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 0.16,
    };

    let mut pb = PathBuilder::stroke(px(1.0));

    // Walk radially outwards in small steps and drop short perpendicular
    // ticks — the effect is a stripe or dotted-stripe pattern visible
    // even in monochrome.
    let base_angle = start + (end - start) * 0.5;
    let segment_count = ((outer - inner.max(0.0)) / step).max(1.0) as i32;
    for i in 0..segment_count {
        let r = inner.max(0.0) + step * (i as f32 + 0.5);
        let (ax, ay) = polar(cx, cy, r, base_angle - 0.05);
        let (bx, by) = polar(cx, cy, r, base_angle + 0.05);
        pb.move_to(point(ax, ay));
        pb.line_to(point(bx, by));
    }

    if let Ok(path) = pb.build() {
        window.paint_path(path, accent);
    }
}

/// Convert polar coordinates to Cartesian `(x, y)` in pixel space.
fn polar(cx: Pixels, cy: Pixels, r: f32, theta: f32) -> (Pixels, Pixels) {
    (cx + px(r * theta.cos()), cy + px(r * theta.sin()))
}

/// Collect (colour, numeric weight) pairs from a [`ChartDataSet`].
///
/// A series contributes its first point's `y` value (sector charts read
/// one scalar per series). Non-numeric values are skipped.
pub(crate) fn sector_weights(
    data_set: &ChartDataSet,
    colors: impl Fn(usize) -> Hsla,
) -> Vec<(Hsla, f32)> {
    data_set
        .series
        .iter()
        .enumerate()
        .filter_map(|(i, series)| {
            let first = series.inner.points.first()?;
            let v = first.y.as_number_f32()?;
            if v <= 0.0 || !v.is_finite() {
                return None;
            }
            Some((colors(i), v))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::hsla;

    use super::*;
    use crate::components::content::chart::types::{ChartDataSeries, ChartSeries};

    fn mk_set(values: &[(&'static str, f32)]) -> ChartDataSet {
        let series: Vec<ChartSeries> = values
            .iter()
            .map(|(n, v)| ChartSeries::new(ChartDataSeries::new(*n, vec![*v])))
            .collect();
        ChartDataSet::multi(series)
    }

    #[test]
    fn weights_extracts_per_series_first_y() {
        let set = mk_set(&[("a", 10.0), ("b", 20.0), ("c", 30.0)]);
        let pairs = sector_weights(&set, |_| hsla(0.0, 0.0, 0.0, 1.0));
        assert_eq!(pairs.len(), 3);
        assert!((pairs[0].1 - 10.0).abs() < 1e-4);
        assert!((pairs[2].1 - 30.0).abs() < 1e-4);
    }

    #[test]
    fn weights_skips_non_positive_and_non_finite() {
        let set = mk_set(&[("a", 10.0), ("b", -5.0), ("c", 0.0), ("d", f32::NAN)]);
        let pairs = sector_weights(&set, |_| hsla(0.0, 0.0, 0.0, 1.0));
        assert_eq!(pairs.len(), 1);
    }

    #[test]
    fn weights_empty_series_is_empty() {
        let set = ChartDataSet::multi(vec![ChartSeries::new(ChartDataSeries::new(
            "empty",
            vec![],
        ))]);
        let pairs = sector_weights(&set, |_| hsla(0.0, 0.0, 0.0, 1.0));
        assert!(pairs.is_empty());
    }

    #[test]
    fn polar_at_zero_points_right() {
        let (x, y) = polar(px(100.0), px(100.0), 10.0, 0.0);
        assert!((f32::from(x) - 110.0).abs() < 1e-4);
        assert!((f32::from(y) - 100.0).abs() < 1e-4);
    }

    #[test]
    fn polar_at_half_pi_points_down() {
        let (x, y) = polar(px(100.0), px(100.0), 10.0, PI / 2.0);
        assert!((f32::from(x) - 100.0).abs() < 1e-4);
        assert!((f32::from(y) - 110.0).abs() < 1e-4);
    }

    #[test]
    fn polar_at_default_start_angle_points_up() {
        let (x, y) = polar(px(100.0), px(100.0), 10.0, DEFAULT_SECTOR_START_ANGLE);
        assert!((f32::from(x) - 100.0).abs() < 1e-4);
        assert!((f32::from(y) - 90.0).abs() < 1e-4);
    }
}
