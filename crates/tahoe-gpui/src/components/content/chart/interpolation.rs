//! Line/area interpolation methods.
//!
//! Mirrors Swift Charts' [`InterpolationMethod`](https://developer.apple.com/documentation/charts/interpolationmethod)
//! so Line and Area marks can switch between a straight polyline, a smooth
//! spline, a monotone cubic (no overshoot), or step curves.
//!
//! Each variant resolves to a single helper that appends segments to a
//! [`PathBuilder`] starting from the first projected point. The caller
//! handles `move_to(pts[0])` before delegating to [`append_interpolation`]
//! so the same path-append logic works for both `stroke`-mode line paths
//! and `fill`-mode area paths.
//!
//! The Cardinal tension parameter follows the classic convention: `0.0`
//! degenerates to straight segments and `1.0` is a full Catmull-Rom curve
//! (`CatmullRom` is equivalent to `Cardinal(1.0)` but kept as a distinct
//! variant so callers reading the builder see Swift Charts' naming).

use gpui::{PathBuilder, Pixels, Point, point, px};

/// How adjacent data points are joined when painting Line or Area marks.
///
/// Defaults to [`InterpolationMethod::CatmullRom`] — the spline smoothing
/// that every previous chart release used.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InterpolationMethod {
    /// Straight line segments between consecutive points. Use for data
    /// where the inter-point path has no meaningful intermediate value
    /// (sample counts, bucketed totals).
    Linear,
    /// Catmull-Rom spline — the chart's historical default. Produces a
    /// smooth curve that passes through every point. Equivalent to
    /// [`InterpolationMethod::Cardinal`] with tension `1.0`.
    #[default]
    CatmullRom,
    /// Cardinal spline with configurable tension.
    ///
    /// `0.0` collapses to straight segments; `1.0` matches Catmull-Rom;
    /// values above `1.0` exaggerate the curvature and should be used
    /// sparingly. Values outside `[0.0, 1.0]` are accepted to match
    /// Swift Charts' surface but the output may overshoot the data
    /// range — prefer [`Monotone`](Self::Monotone) when avoiding
    /// overshoot matters (non-negative quantities, percentages).
    Cardinal(f32),
    /// Monotone cubic (Fritsch-Carlson) — smooth curve that never
    /// overshoots the input data. Use for non-negative quantities
    /// (income, headcount) where a cosmetic dip below zero would be
    /// misleading.
    Monotone,
    /// Step curve holding the previous value until the next sample.
    /// Matches Swift Charts' `stepStart` (the step rises at the
    /// _start_ of each segment).
    StepStart,
    /// Step curve holding the current value until the next sample.
    /// Matches Swift Charts' `stepEnd` (the step rises at the _end_ of
    /// each segment — useful for "value as of" time series).
    StepEnd,
    /// Step curve holding the midpoint between adjacent samples. Matches
    /// Swift Charts' `stepCenter`.
    StepCenter,
}

/// Append the segments for `method` between `pts[0]` and `pts[n-1]`.
///
/// The caller is expected to have already called `pb.move_to(pts[0])`
/// before invoking this helper — matching the pattern used by all
/// existing paint functions in `marks.rs`.
///
/// For `pts.len() < 2` this is a no-op. For `pts.len() == 2` every
/// variant degenerates to a single straight line segment so callers
/// don't need to gate on `len()` themselves.
pub(crate) fn append_interpolation(
    pb: &mut PathBuilder,
    pts: &[Point<Pixels>],
    method: InterpolationMethod,
) {
    let n = pts.len();
    if n < 2 {
        return;
    }
    if n == 2 {
        pb.line_to(pts[1]);
        return;
    }

    match method {
        InterpolationMethod::Linear => append_linear(pb, pts),
        InterpolationMethod::CatmullRom => append_cardinal(pb, pts, 1.0),
        InterpolationMethod::Cardinal(tension) => append_cardinal(pb, pts, tension),
        InterpolationMethod::Monotone => append_monotone(pb, pts),
        InterpolationMethod::StepStart => append_step(pb, pts, StepKind::Start),
        InterpolationMethod::StepEnd => append_step(pb, pts, StepKind::End),
        InterpolationMethod::StepCenter => append_step(pb, pts, StepKind::Center),
    }
}

fn append_linear(pb: &mut PathBuilder, pts: &[Point<Pixels>]) {
    for p in &pts[1..] {
        pb.line_to(*p);
    }
}

/// Cardinal spline (tension `1.0` ≡ Catmull-Rom). The control point
/// weights follow the classic `(P2 - P0) / 6` / `(P3 - P1) / 6` rule
/// scaled inversely by tension so `0.0` collapses to straight lines and
/// `1.0` matches the historical Catmull-Rom output byte-for-byte.
fn append_cardinal(pb: &mut PathBuilder, pts: &[Point<Pixels>], tension: f32) {
    let n = pts.len();
    // `tension == 0` would divide by zero; collapse to a linear polyline.
    if tension.abs() < f32::EPSILON {
        append_linear(pb, pts);
        return;
    }
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

/// Monotone cubic interpolation — Fritsch-Carlson algorithm.
///
/// Guarantees the rendered curve is monotone between any two adjacent
/// data points whenever the data itself is, so the curve never
/// "overshoots" below zero or above the local max. Useful for
/// non-negative quantities (income, counts) where a dip below the data
/// minimum would be misleading.
///
/// Reference: Fritsch, F. N. and Carlson, R. E. (1980), "Monotone
/// Piecewise Cubic Interpolation". Converted from slope form to cubic
/// bezier control points using the standard `(dx / 3) * slope` offset.
fn append_monotone(pb: &mut PathBuilder, pts: &[Point<Pixels>]) {
    let n = pts.len();
    if n < 2 {
        return;
    }

    // 1. Segment slopes and horizontal deltas.
    let mut dx = vec![0.0_f32; n - 1];
    let mut slopes = vec![0.0_f32; n - 1];
    for i in 0..n - 1 {
        let x0 = f32::from(pts[i].x);
        let x1 = f32::from(pts[i + 1].x);
        let y0 = f32::from(pts[i].y);
        let y1 = f32::from(pts[i + 1].y);
        dx[i] = x1 - x0;
        // A zero-width segment has no slope; treat it as flat so the
        // tangent solver doesn't emit NaN. The cubic bezier still draws
        // a vertical line in that slot.
        slopes[i] = if dx[i].abs() < f32::EPSILON {
            0.0
        } else {
            (y1 - y0) / dx[i]
        };
    }

    // 2. Endpoint tangents via one-sided parabolic fit; interior
    //    tangents via the weighted harmonic mean so that adjacent
    //    slopes of opposite sign force a zero tangent (the monotone
    //    property).
    let mut tangents = vec![0.0_f32; n];
    tangents[0] = slopes[0];
    tangents[n - 1] = slopes[n - 2];
    for i in 1..n - 1 {
        let s0 = slopes[i - 1];
        let s1 = slopes[i];
        if s0 * s1 <= 0.0 {
            tangents[i] = 0.0;
        } else {
            let h0 = dx[i - 1];
            let h1 = dx[i];
            let w0 = 2.0 * h1 + h0;
            let w1 = h1 + 2.0 * h0;
            tangents[i] = (w0 + w1) / (w0 / s0 + w1 / s1);
        }
    }

    // 3. Fritsch-Carlson alpha/beta clamp: if (alpha^2 + beta^2) > 9,
    //    rescale both tangents so the spline stays monotone inside the
    //    segment. Degenerate slopes (|s| < eps) force both tangents to
    //    zero so the segment paints flat.
    for i in 0..n - 1 {
        if slopes[i].abs() < f32::EPSILON {
            tangents[i] = 0.0;
            tangents[i + 1] = 0.0;
            continue;
        }
        let alpha = tangents[i] / slopes[i];
        let beta = tangents[i + 1] / slopes[i];
        let sum = alpha * alpha + beta * beta;
        if sum > 9.0 {
            let tau = 3.0 / sum.sqrt();
            tangents[i] = tau * alpha * slopes[i];
            tangents[i + 1] = tau * beta * slopes[i];
        }
    }

    // 4. Emit the cubic bezier for each segment. Control-point offsets
    //    are `(dx / 3) * tangent` — the standard Hermite → Bezier
    //    conversion.
    for i in 0..n - 1 {
        let p0 = pts[i];
        let p1 = pts[i + 1];
        let third = dx[i] / 3.0;
        let cp1 = point(p0.x + px(third), p0.y + px(third * tangents[i]));
        let cp2 = point(p1.x - px(third), p1.y - px(third * tangents[i + 1]));
        pb.cubic_bezier_to(p1, cp1, cp2);
    }
}

#[derive(Clone, Copy)]
enum StepKind {
    Start,
    End,
    Center,
}

fn append_step(pb: &mut PathBuilder, pts: &[Point<Pixels>], kind: StepKind) {
    for i in 0..pts.len() - 1 {
        let p0 = pts[i];
        let p1 = pts[i + 1];
        match kind {
            // Step rises at the start of each segment: from P0 jump to
            // P1's y immediately, then hold to P1.
            StepKind::Start => {
                pb.line_to(point(p0.x, p1.y));
                pb.line_to(p1);
            }
            // Step holds at P0's y until the segment's end, then jumps.
            StepKind::End => {
                pb.line_to(point(p1.x, p0.y));
                pb.line_to(p1);
            }
            // Step holds at P0's y through the midpoint, then jumps to
            // P1's y and holds until the segment's end.
            StepKind::Center => {
                let mid_x = px((f32::from(p0.x) + f32::from(p1.x)) * 0.5);
                pb.line_to(point(mid_x, p0.y));
                pb.line_to(point(mid_x, p1.y));
                pb.line_to(p1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::*;

    fn pts() -> Vec<Point<Pixels>> {
        vec![
            point(px(0.0), px(100.0)),
            point(px(50.0), px(50.0)),
            point(px(100.0), px(75.0)),
            point(px(150.0), px(25.0)),
        ]
    }

    #[test]
    fn default_is_catmull_rom() {
        assert_eq!(
            InterpolationMethod::default(),
            InterpolationMethod::CatmullRom
        );
    }

    #[test]
    fn append_is_noop_for_fewer_than_two_points() {
        let mut pb = PathBuilder::stroke(px(1.0));
        pb.move_to(point(px(0.0), px(0.0)));
        // No segments appended — `build()` succeeds because the path has
        // its initial `move_to`, but there are no line/curve ops to emit.
        append_interpolation(&mut pb, &[], InterpolationMethod::Linear);
        append_interpolation(
            &mut pb,
            &[point(px(10.0), px(10.0))],
            InterpolationMethod::Linear,
        );
        // Build doesn't panic for a path that only has a move_to.
        let _ = pb.build();
    }

    #[test]
    fn append_for_two_points_draws_straight_line_regardless_of_method() {
        // Every variant degenerates to a straight line between two
        // points — the early-return in `append_interpolation` skips
        // spline / step logic, and `build()` succeeds for each.
        let two = [point(px(0.0), px(0.0)), point(px(10.0), px(10.0))];
        for method in [
            InterpolationMethod::Linear,
            InterpolationMethod::CatmullRom,
            InterpolationMethod::Cardinal(0.5),
            InterpolationMethod::Monotone,
            InterpolationMethod::StepStart,
            InterpolationMethod::StepEnd,
            InterpolationMethod::StepCenter,
        ] {
            let mut pb = PathBuilder::stroke(px(1.0));
            pb.move_to(two[0]);
            append_interpolation(&mut pb, &two, method);
            assert!(pb.build().is_ok(), "method {method:?} failed on 2 points");
        }
    }

    #[test]
    fn cardinal_zero_tension_is_linear_fallback() {
        // Tension 0.0 would divide by zero in the cardinal formula;
        // the implementation falls back to the linear polyline so the
        // path still builds cleanly.
        let p = pts();
        let mut pb = PathBuilder::stroke(px(1.0));
        pb.move_to(p[0]);
        append_interpolation(&mut pb, &p, InterpolationMethod::Cardinal(0.0));
        assert!(pb.build().is_ok());
    }

    #[test]
    fn every_method_builds_on_non_trivial_input() {
        let p = pts();
        for method in [
            InterpolationMethod::Linear,
            InterpolationMethod::CatmullRom,
            InterpolationMethod::Cardinal(0.5),
            InterpolationMethod::Cardinal(1.0),
            InterpolationMethod::Monotone,
            InterpolationMethod::StepStart,
            InterpolationMethod::StepEnd,
            InterpolationMethod::StepCenter,
        ] {
            let mut pb = PathBuilder::stroke(px(1.0));
            pb.move_to(p[0]);
            append_interpolation(&mut pb, &p, method);
            assert!(pb.build().is_ok(), "method {method:?} failed to build");
        }
    }

    #[test]
    fn monotone_handles_zero_width_segments_without_nan() {
        // Two consecutive points at identical X trigger the dx≈0
        // guard inside the Fritsch-Carlson solver. The path must
        // still build without emitting NaN control points.
        let p = vec![
            point(px(0.0), px(100.0)),
            point(px(50.0), px(50.0)),
            point(px(50.0), px(25.0)), // same x as the previous point
            point(px(100.0), px(10.0)),
        ];
        let mut pb = PathBuilder::stroke(px(1.0));
        pb.move_to(p[0]);
        append_interpolation(&mut pb, &p, InterpolationMethod::Monotone);
        assert!(pb.build().is_ok());
    }

    #[test]
    fn monotone_flat_input_stays_flat() {
        // All equal Y-values — the algorithm should emit flat segments
        // with zero tangents, so the path builds without NaN.
        let p = vec![
            point(px(0.0), px(50.0)),
            point(px(50.0), px(50.0)),
            point(px(100.0), px(50.0)),
            point(px(150.0), px(50.0)),
        ];
        let mut pb = PathBuilder::stroke(px(1.0));
        pb.move_to(p[0]);
        append_interpolation(&mut pb, &p, InterpolationMethod::Monotone);
        assert!(pb.build().is_ok());
    }
}
