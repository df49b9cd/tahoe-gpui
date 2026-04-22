//! Rectangle / heatmap mark renderer.
//!
//! Each data point's `(x, y)` selects a cell in a grid whose axes are
//! driven by the chart's X and Y scales. The magnitude comes from the
//! optional `z` channel — when `z` is absent we fall back to `y` so a
//! two-channel point still plots meaningfully.
//!
//! Cells are painted as filled rectangles with colour lightness
//! interpolated between the base colour's lightness and a near-white
//! ceiling; when Differentiate-Without-Color is active each cell also
//! receives a 1-pt border so magnitude contrast is readable in
//! monochrome.
//!
//! # HIG
//!
//! <https://developer.apple.com/design/human-interface-guidelines/charts>

use gpui::{Hsla, PathBuilder, Pixels, Point, Window, point, px};

use super::scales::Scale;
use super::types::{ChartDataSet, PlottableValue};

/// Resolved heatmap cell geometry — one per plotted (x, y, z) triple.
#[derive(Clone, Copy)]
pub(crate) struct HeatmapCell {
    pub x: Pixels,
    pub y: Pixels,
    pub w: Pixels,
    pub h: Pixels,
    pub magnitude: f32,
    /// Source series index (for FKA label/handle lookup).
    pub series_index: usize,
    /// Slot within the source series (for FKA label/handle lookup).
    pub slot_index: usize,
}

/// Paint a heatmap across the plot rectangle.
///
/// Cells sized off the scale's projected span. Magnitudes below `z_min`
/// paint near-white; magnitudes at `z_max` paint the base colour.
pub(crate) fn paint_rectangle_chart(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    cells: &[HeatmapCell],
    z_min: f32,
    z_max: f32,
    base_color: Hsla,
    dwc: bool,
) {
    if cells.is_empty() || w <= 0.0 || h <= 0.0 {
        return;
    }

    let z_span = (z_max - z_min).max(f32::EPSILON);
    for cell in cells {
        let t = ((cell.magnitude - z_min) / z_span).clamp(0.0, 1.0);
        let color = lerp_lightness(base_color, 0.95, t);

        // Fill the cell.
        let mut pb = PathBuilder::fill();
        let x0 = origin.x + cell.x;
        let y0 = origin.y + cell.y;
        let x1 = x0 + cell.w;
        let y1 = y0 + cell.h;
        pb.move_to(point(x0, y0));
        pb.line_to(point(x1, y0));
        pb.line_to(point(x1, y1));
        pb.line_to(point(x0, y1));
        pb.close();
        if let Ok(path) = pb.build() {
            window.paint_path(path, color);
        }

        if dwc {
            let border = Hsla {
                l: 0.0,
                a: 0.3,
                ..base_color
            };
            let mut pb = PathBuilder::stroke(px(1.0));
            pb.move_to(point(x0, y0));
            pb.line_to(point(x1, y0));
            pb.line_to(point(x1, y1));
            pb.line_to(point(x0, y1));
            pb.close();
            if let Ok(path) = pb.build() {
                window.paint_path(path, border);
            }
        }
    }

    let _ = w;
    let _ = h;
}

/// Linear interpolation along lightness. `t == 0.0` returns the full
/// base colour; `t == 1.0` pulls lightness towards `target_lightness`.
fn lerp_lightness(base: Hsla, target_lightness: f32, t: f32) -> Hsla {
    // `t` runs magnitude-low to magnitude-high; high magnitude keeps
    // base colour, low magnitude washes out towards `target_lightness`.
    let l = target_lightness * (1.0 - t) + base.l * t;
    Hsla {
        l: l.clamp(0.0, 1.0),
        ..base
    }
}

/// Build cell geometry from a data set using the caller's scales.
///
/// Each cell occupies one slot of the X scale's domain × one slot of the
/// Y scale's domain. The slot width comes from the scale density (we
/// assume evenly-spaced ticks for a CategoryScale and the full domain
/// for numeric scales). Non-numeric `z` values fall back to the
/// numeric `y`; points where neither is numeric are skipped.
pub(crate) fn build_cells(
    data_set: &ChartDataSet,
    x_scale: &dyn Scale,
    y_scale: &dyn Scale,
    w: f32,
    h: f32,
    x_slots: usize,
    y_slots: usize,
) -> (Vec<HeatmapCell>, f32, f32) {
    let cell_w = if x_slots == 0 { w } else { w / x_slots as f32 };
    let cell_h = if y_slots == 0 { h } else { h / y_slots as f32 };
    let mut cells = Vec::new();
    let mut z_min = f32::INFINITY;
    let mut z_max = f32::NEG_INFINITY;

    for (si, series) in data_set.series.iter().enumerate() {
        for (slot_i, p) in series.inner.points.iter().enumerate() {
            // Heatmap magnitude priority:
            //   1. `z` when numeric (the documented, intended input);
            //   2. `y` when `z` is absent or non-numeric (fallback for
            //      sparse data sources);
            //   3. skip the cell when neither is numeric.
            // Surface a debug assertion when `z` is *present but the
            // wrong type* — that almost always indicates a data-source
            // bug (e.g. passing a `Date` into the magnitude axis) that
            // would otherwise render as an invisible cell.
            let magnitude = match &p.z {
                Some(PlottableValue::Number(n)) => *n as f32,
                Some(_other) => {
                    debug_assert!(
                        false,
                        "rectangle chart: `z` must be numeric to drive magnitude; \
                         got {:?}. Falling back to `y`.",
                        _other
                    );
                    match p.y.as_number_f32() {
                        Some(v) => v,
                        None => continue,
                    }
                }
                None => match p.y.as_number_f32() {
                    Some(v) => v,
                    None => continue,
                },
            };
            z_min = z_min.min(magnitude);
            z_max = z_max.max(magnitude);

            let xn = x_scale.project(&p.x).clamp(0.0, 1.0);
            let yn = y_scale.project(&p.y).clamp(0.0, 1.0);
            let x_px = px(xn * w - cell_w * 0.5);
            let y_px = px((1.0 - yn) * h - cell_h * 0.5);
            cells.push(HeatmapCell {
                x: x_px,
                y: y_px,
                w: px(cell_w),
                h: px(cell_h),
                magnitude,
                series_index: si,
                slot_index: slot_i,
            });
        }
    }

    if cells.is_empty() {
        (cells, 0.0, 1.0)
    } else {
        (cells, z_min, z_max)
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::hsla;

    use super::*;
    use crate::components::content::chart::scales::LinearScale;
    use crate::components::content::chart::types::{ChartDataSeries, ChartPoint, ChartSeries};

    fn mk_cell_set(points: Vec<ChartPoint>) -> ChartDataSet {
        let series = ChartDataSeries::from_points("grid", points);
        ChartDataSet::multi(vec![ChartSeries::new(series)])
    }

    #[test]
    fn empty_dataset_produces_no_cells() {
        let set = mk_cell_set(vec![]);
        let xs = LinearScale::new(0.0, 1.0);
        let ys = LinearScale::new(0.0, 1.0);
        let (cells, _lo, _hi) = build_cells(&set, &xs, &ys, 100.0, 100.0, 1, 1);
        assert!(cells.is_empty());
    }

    #[test]
    fn magnitude_prefers_z_over_y() {
        let set = mk_cell_set(vec![
            ChartPoint::new(0.0, 1.0).with_z(42.0),
            ChartPoint::new(1.0, 1.0), // no z — falls back to y (1.0)
        ]);
        let xs = LinearScale::new(0.0, 1.0);
        let ys = LinearScale::new(0.0, 1.0);
        let (cells, lo, hi) = build_cells(&set, &xs, &ys, 100.0, 100.0, 2, 1);
        assert_eq!(cells.len(), 2);
        assert!((lo - 1.0).abs() < 1e-4);
        assert!((hi - 42.0).abs() < 1e-4);
    }

    #[test]
    fn lerp_lightness_at_t1_returns_base() {
        let base = hsla(0.5, 0.5, 0.3, 1.0);
        let out = lerp_lightness(base, 0.95, 1.0);
        assert!((out.l - 0.3).abs() < 1e-4);
    }

    #[test]
    fn lerp_lightness_at_t0_returns_target() {
        let base = hsla(0.5, 0.5, 0.3, 1.0);
        let out = lerp_lightness(base, 0.95, 0.0);
        assert!((out.l - 0.95).abs() < 1e-4);
    }
}
