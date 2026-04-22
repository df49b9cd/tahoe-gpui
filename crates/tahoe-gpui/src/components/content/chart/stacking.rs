//! Mark stacking — Swift Charts' `MarkStackingMethod` translated for
//! `tahoe-gpui`. Stacking computes per-slot `(lo, hi)` value pairs for
//! every series so bars and areas stack positively (Standard), normalise
//! to a per-slot total of 1.0 (Normalized), centre on a meandering
//! baseline (Center), or fall back to the current overlay behaviour
//! (Unstacked).
//!
//! A single helper [`compute_stacks`] produces the lookup `stacks[si]
//! [slot]` so both `render_bars` and the Area branch of `render_canvas`
//! can read positional data without reimplementing the math.
//!
//! # HIG
//!
//! <https://developer.apple.com/design/human-interface-guidelines/charts>

use super::types::ChartDataSet;

/// How marks accumulate when more than one series shares a slot.
///
/// Mirrors Swift Charts' [`MarkStackingMethod`](https://developer.apple.com/documentation/charts/markstackingmethod).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkStackingMethod {
    /// Each slot's marks are overlaid (no stacking math). For Bar charts,
    /// this draws series side-by-side inside the slot — the v1
    /// behaviour.
    #[default]
    Unstacked,
    /// Marks stack positively from the baseline. Each series sits on top
    /// of the cumulative sum of all preceding series in the slot.
    Standard,
    /// Per-slot total is normalised to `1.0`; each series occupies a
    /// proportional band. Hides absolute magnitude, exposes ratios.
    Normalized,
    /// Stream-graph centring: each slot's stack is offset so its midpoint
    /// sits at `0.5`. Useful when relative size matters more than the
    /// absolute baseline.
    Center,
}

impl MarkStackingMethod {
    /// Whether the method changes per-series positioning at all. `Unstacked`
    /// is a no-op so the renderer can take a fast path.
    pub(crate) fn is_active(self) -> bool {
        !matches!(self, MarkStackingMethod::Unstacked)
    }
}

/// One stack segment per (series, slot) pair, in normalised `[0, 1]`
/// coordinates ready to be projected against the plot height.
#[derive(Debug, Clone, Copy)]
pub(crate) struct StackSegment {
    pub lo: f32,
    pub hi: f32,
}

impl StackSegment {
    /// Convenience for the common `[lo, lo]` empty segment used when a
    /// series has no value at a slot — the renderer simply paints
    /// nothing.
    pub(crate) fn empty(at: f32) -> Self {
        Self { lo: at, hi: at }
    }
}

/// Compute per-(series, slot) stacking segments in `[0, 1]` space.
///
/// `data_min` / `data_max` come from the chart's data extent — they
/// frame the linear projection used by `Standard` and `Center` so a
/// negative-only data set still fills the plot. `Normalized` ignores
/// them: every slot's total is forced to 1.0 regardless of the source
/// magnitudes.
///
/// Returns a `Vec<Vec<StackSegment>>` shaped `[series_idx][slot_idx]`.
/// Slots beyond a series' own length collapse to an empty segment so
/// ragged multi-series stacks compose without panicking.
pub(crate) fn compute_stacks(
    data_set: &ChartDataSet,
    method: MarkStackingMethod,
    data_min: f32,
    data_max: f32,
) -> Vec<Vec<StackSegment>> {
    let n_series = data_set.series.len();
    let max_slots = data_set.max_points().max(1);
    let mut segments = vec![vec![StackSegment::empty(0.0); max_slots]; n_series];

    if !method.is_active() {
        return segments;
    }

    let span = (data_max - data_min).max(f32::EPSILON);
    let to_norm = |v: f32| ((v - data_min) / span).clamp(0.0, 1.0);

    // Per-slot totals only needed by Normalized & Center.
    let totals: Vec<f32> = (0..max_slots)
        .map(|slot| {
            let mut sum = 0.0;
            for series in data_set.series.iter() {
                if let Some(p) = series.inner.points.get(slot)
                    && let Some(v) = p.y.as_number_f32()
                    && v.is_finite()
                    && v >= 0.0
                {
                    sum += v;
                }
            }
            sum
        })
        .collect();

    for slot in 0..max_slots {
        match method {
            MarkStackingMethod::Unstacked => unreachable!("filtered above"),
            MarkStackingMethod::Standard => {
                let mut cursor = to_norm(0.0_f32.max(data_min));
                for (si, series) in data_set.series.iter().enumerate() {
                    let Some(p) = series.inner.points.get(slot) else {
                        segments[si][slot] = StackSegment::empty(cursor);
                        continue;
                    };
                    let v = p.y.as_number_f32().unwrap_or(0.0);
                    if v < 0.0 {
                        // Negative values would underflow the stack;
                        // treat as zero to keep stacks anchored at the
                        // baseline.
                        segments[si][slot] = StackSegment::empty(cursor);
                        continue;
                    }
                    let next = (cursor + v / span).clamp(0.0, 1.0);
                    segments[si][slot] = StackSegment {
                        lo: cursor,
                        hi: next,
                    };
                    cursor = next;
                }
            }
            MarkStackingMethod::Normalized => {
                let total = totals[slot].max(f32::EPSILON);
                let mut cursor = 0.0;
                for (si, series) in data_set.series.iter().enumerate() {
                    let Some(p) = series.inner.points.get(slot) else {
                        segments[si][slot] = StackSegment::empty(cursor);
                        continue;
                    };
                    let v = p.y.as_number_f32().unwrap_or(0.0).max(0.0);
                    let next = (cursor + v / total).clamp(0.0, 1.0);
                    segments[si][slot] = StackSegment {
                        lo: cursor,
                        hi: next,
                    };
                    cursor = next;
                }
            }
            MarkStackingMethod::Center => {
                let total = totals[slot];
                if total <= 0.0 {
                    continue;
                }
                let total_norm = (total / span).clamp(0.0, 1.0);
                let mut cursor = (1.0 - total_norm) * 0.5;
                for (si, series) in data_set.series.iter().enumerate() {
                    let Some(p) = series.inner.points.get(slot) else {
                        segments[si][slot] = StackSegment::empty(cursor);
                        continue;
                    };
                    let v = p.y.as_number_f32().unwrap_or(0.0).max(0.0);
                    let next = (cursor + v / span).clamp(0.0, 1.0);
                    segments[si][slot] = StackSegment {
                        lo: cursor,
                        hi: next,
                    };
                    cursor = next;
                }
            }
        }
    }

    segments
}

/// Per-slot cumulative total — used by tooltips so a hover read-out can
/// show both the segment value and the running stack height. Returns
/// `None` for [`MarkStackingMethod::Unstacked`] since totals aren't
/// meaningful when the marks overlay.
#[allow(dead_code)] // wired in Phase 9 (selection binding) tooltips.
pub(crate) fn per_slot_totals(data_set: &ChartDataSet) -> Vec<f32> {
    let max_slots = data_set.max_points().max(1);
    (0..max_slots)
        .map(|slot| {
            let mut sum = 0.0;
            for series in data_set.series.iter() {
                if let Some(p) = series.inner.points.get(slot)
                    && let Some(v) = p.y.as_number_f32()
                    && v.is_finite()
                    && v >= 0.0
                {
                    sum += v;
                }
            }
            sum
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::*;
    use crate::components::content::chart::types::{ChartDataSeries, ChartSeries};

    fn mk_set(series: Vec<(&'static str, Vec<f32>)>) -> ChartDataSet {
        ChartDataSet::multi(
            series
                .into_iter()
                .map(|(n, vs)| ChartSeries::new(ChartDataSeries::new(n, vs)))
                .collect(),
        )
    }

    #[test]
    fn unstacked_returns_zero_segments() {
        let set = mk_set(vec![("a", vec![1.0, 2.0]), ("b", vec![3.0, 4.0])]);
        let stacks = compute_stacks(&set, MarkStackingMethod::Unstacked, 0.0, 10.0);
        for series in stacks {
            for seg in series {
                assert!(seg.hi == seg.lo);
            }
        }
    }

    #[test]
    fn standard_stacks_positively() {
        let set = mk_set(vec![("a", vec![10.0, 20.0]), ("b", vec![30.0, 40.0])]);
        let stacks = compute_stacks(&set, MarkStackingMethod::Standard, 0.0, 100.0);
        // Slot 0: a 0..10, b 10..40
        assert!((stacks[0][0].lo - 0.0).abs() < 1e-4);
        assert!((stacks[0][0].hi - 0.10).abs() < 1e-4);
        assert!((stacks[1][0].lo - 0.10).abs() < 1e-4);
        assert!((stacks[1][0].hi - 0.40).abs() < 1e-4);
        // Slot 1: a 0..20, b 20..60
        assert!((stacks[0][1].hi - 0.20).abs() < 1e-4);
        assert!((stacks[1][1].hi - 0.60).abs() < 1e-4);
    }

    #[test]
    fn normalized_each_slot_sums_to_one() {
        let set = mk_set(vec![("a", vec![10.0, 20.0]), ("b", vec![30.0, 60.0])]);
        let stacks = compute_stacks(&set, MarkStackingMethod::Normalized, 0.0, 100.0);
        // Slot 0: a 0..0.25, b 0.25..1.0
        assert!((stacks[0][0].hi - 0.25).abs() < 1e-4);
        assert!((stacks[1][0].hi - 1.0).abs() < 1e-4);
        // Slot 1: a 0..0.25, b 0.25..1.0
        assert!((stacks[0][1].hi - 0.25).abs() < 1e-4);
        assert!((stacks[1][1].hi - 1.0).abs() < 1e-4);
    }

    #[test]
    fn center_balances_around_midline() {
        let set = mk_set(vec![("a", vec![10.0]), ("b", vec![10.0])]);
        let stacks = compute_stacks(&set, MarkStackingMethod::Center, 0.0, 100.0);
        // Total = 20 / span 100 = 0.20. Cursor starts at (1 - 0.20)/2 = 0.40.
        // a 0.40..0.50, b 0.50..0.60
        assert!((stacks[0][0].lo - 0.40).abs() < 1e-4);
        assert!((stacks[0][0].hi - 0.50).abs() < 1e-4);
        assert!((stacks[1][0].lo - 0.50).abs() < 1e-4);
        assert!((stacks[1][0].hi - 0.60).abs() < 1e-4);
    }

    #[test]
    fn ragged_series_collapses_to_empty_segment() {
        let set = mk_set(vec![("a", vec![10.0, 20.0]), ("b", vec![30.0])]);
        let stacks = compute_stacks(&set, MarkStackingMethod::Standard, 0.0, 100.0);
        // Slot 1 has only a — b should be an empty segment at the cursor
        // after a (`0.20`).
        assert!((stacks[1][1].lo - stacks[1][1].hi).abs() < 1e-4);
        assert!((stacks[1][1].lo - 0.20).abs() < 1e-4);
    }

    #[test]
    fn negative_values_dont_underflow_the_stack() {
        let set = mk_set(vec![("a", vec![-10.0]), ("b", vec![20.0])]);
        let stacks = compute_stacks(&set, MarkStackingMethod::Standard, 0.0, 100.0);
        // Negative value collapses to empty segment so b still anchors at zero.
        assert!((stacks[0][0].lo - stacks[0][0].hi).abs() < 1e-4);
        assert!((stacks[1][0].lo - 0.0).abs() < 1e-4);
        assert!((stacks[1][0].hi - 0.20).abs() < 1e-4);
    }

    #[test]
    fn per_slot_totals_skips_negative_and_non_numeric() {
        let set = mk_set(vec![("a", vec![10.0, -5.0]), ("b", vec![20.0, 30.0])]);
        let totals = per_slot_totals(&set);
        assert!((totals[0] - 30.0).abs() < 1e-4);
        // Slot 1: -5 skipped, 30 counted.
        assert!((totals[1] - 30.0).abs() < 1e-4);
    }
}
