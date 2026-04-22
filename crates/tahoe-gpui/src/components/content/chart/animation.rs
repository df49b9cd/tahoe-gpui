//! Animated data transitions for [`super::ChartView`].
//!
//! Mirrors Apple's Swift Charts behaviour where replacing a
//! [`ChartDataSet`] springs each mark between its old and new position
//! over ~300 ms. The duration lines up with the HIG medium ramp
//! (`MotionRamp::Medium`, matching SwiftUI's implicit `.spring`).
//!
//! Date and Category Y values cross-fade at the midpoint — a calendar
//! instant or a string label has no meaningful interpolation — while
//! numeric Y values lerp. When `REDUCE_MOTION` is set on the active
//! theme, [`super::ChartView::set_data`] snaps to the new state instead
//! of scheduling any per-frame ticks (HIG: "replace large, dramatic
//! transitions with subtle cross-fades").

use std::time::Duration;

use super::types::{ChartDataSeries, ChartDataSet, ChartPoint, ChartSeries, PlottableValue};
use crate::foundations::motion::MotionRamp;

/// Default transition duration when calling
/// [`super::ChartView::set_data`].
///
/// Matches the HIG medium ramp (`MotionRamp::Medium`, ~300 ms) — the
/// same window SwiftUI's implicit `.spring` settles over for
/// `response=0.55`.
pub(crate) const DATA_TRANSITION_DURATION: Duration =
    Duration::from_millis(MotionRamp::Medium.duration_ms());

/// Blend two data sets at `progress` in `[0.0, 1.0]`.
///
/// - `progress <= 0.0` returns a clone of `prev`.
/// - `progress >= 1.0` returns a clone of `next`.
/// - Numeric Y / Y-high / Z values lerp.
/// - Non-numeric (`Date`, `Category`) values cross-fade at `progress >=
///   0.5` — there's no meaningful intermediate value between two dates
///   or category labels.
/// - X values always come from `next` so axis labels never flicker
///   mid-tween; the axis domain is a function of the target data, not
///   the source.
/// - Series and points pair by index. Points that exist in only one
///   side appear/disappear at the midpoint rather than tweening.
pub(crate) fn interpolate_data_set(
    prev: &ChartDataSet,
    next: &ChartDataSet,
    progress: f32,
) -> ChartDataSet {
    let t = progress.clamp(0.0, 1.0);
    if t <= 0.0 {
        return prev.clone();
    }
    if t >= 1.0 {
        return next.clone();
    }

    let crossfade = t >= 0.5;
    let series_count = prev.series.len().max(next.series.len());
    let mut out = Vec::with_capacity(series_count);
    for si in 0..series_count {
        match (prev.series.get(si), next.series.get(si)) {
            (Some(p), Some(n)) => {
                let points = interpolate_points(&p.inner.points, &n.inner.points, t, crossfade);
                out.push(ChartSeries {
                    inner: ChartDataSeries::from_points(n.inner.name.clone(), points),
                    color: n.color.or(p.color),
                });
            }
            // Series only exists in one side — cross-fade it in/out at
            // the midpoint so we never paint ghost series during the
            // first half of the tween.
            (None, Some(n)) if crossfade => out.push(n.clone()),
            (Some(p), None) if !crossfade => out.push(p.clone()),
            _ => {}
        }
    }
    ChartDataSet::multi(out)
}

fn interpolate_points(
    prev_points: &[ChartPoint],
    next_points: &[ChartPoint],
    t: f32,
    crossfade: bool,
) -> Vec<ChartPoint> {
    let count = prev_points.len().max(next_points.len());
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        match (prev_points.get(i), next_points.get(i)) {
            (Some(p), Some(n)) => {
                out.push(ChartPoint {
                    x: n.x.clone(),
                    y: lerp_value(&p.y, &n.y, t, crossfade),
                    y_high: match (&p.y_high, &n.y_high) {
                        (Some(a), Some(b)) => Some(lerp_value(a, b, t, crossfade)),
                        (_, Some(b)) if crossfade => Some(b.clone()),
                        (Some(a), _) if !crossfade => Some(a.clone()),
                        _ => None,
                    },
                    z: match (&p.z, &n.z) {
                        (Some(a), Some(b)) => Some(lerp_value(a, b, t, crossfade)),
                        (_, Some(b)) if crossfade => Some(b.clone()),
                        (Some(a), _) if !crossfade => Some(a.clone()),
                        _ => None,
                    },
                });
            }
            (None, Some(n)) if crossfade => out.push(n.clone()),
            (Some(p), None) if !crossfade => out.push(p.clone()),
            _ => {}
        }
    }
    out
}

fn lerp_value(a: &PlottableValue, b: &PlottableValue, t: f32, crossfade: bool) -> PlottableValue {
    match (a, b) {
        (PlottableValue::Number(x), PlottableValue::Number(y)) => {
            let tf = t as f64;
            PlottableValue::Number(x + (y - x) * tf)
        }
        // Dates and categories don't interpolate — snap at the midpoint.
        _ => {
            if crossfade {
                b.clone()
            } else {
                a.clone()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;
    use std::time::SystemTime;

    use super::{DATA_TRANSITION_DURATION, interpolate_data_set};
    use crate::components::content::chart::types::{
        ChartDataSeries, ChartDataSet, ChartPoint, ChartSeries, PlottableValue,
    };

    fn number_set(name: &str, values: &[f32]) -> ChartDataSet {
        ChartDataSet::single(ChartDataSeries::new(name.to_string(), values.to_vec()))
    }

    fn nth_y(set: &ChartDataSet, si: usize, pi: usize) -> f32 {
        set.series[si].inner.points[pi]
            .y
            .as_number_f32()
            .expect("numeric y")
    }

    #[test]
    fn transition_duration_matches_motion_medium_ramp() {
        use crate::foundations::motion::MotionRamp;
        assert_eq!(
            DATA_TRANSITION_DURATION.as_millis() as u64,
            MotionRamp::Medium.duration_ms()
        );
    }

    #[test]
    fn progress_zero_returns_prev() {
        let a = number_set("S", &[10.0, 20.0]);
        let b = number_set("S", &[30.0, 40.0]);
        let blended = interpolate_data_set(&a, &b, 0.0);
        assert_eq!(nth_y(&blended, 0, 0), 10.0);
        assert_eq!(nth_y(&blended, 0, 1), 20.0);
    }

    #[test]
    fn progress_one_returns_next() {
        let a = number_set("S", &[10.0, 20.0]);
        let b = number_set("S", &[30.0, 40.0]);
        let blended = interpolate_data_set(&a, &b, 1.0);
        assert_eq!(nth_y(&blended, 0, 0), 30.0);
        assert_eq!(nth_y(&blended, 0, 1), 40.0);
    }

    #[test]
    fn progress_half_lerps_numeric_y() {
        let a = number_set("S", &[10.0, 20.0]);
        let b = number_set("S", &[30.0, 40.0]);
        let blended = interpolate_data_set(&a, &b, 0.5);
        assert!((nth_y(&blended, 0, 0) - 20.0).abs() < 1e-4);
        assert!((nth_y(&blended, 0, 1) - 30.0).abs() < 1e-4);
    }

    #[test]
    fn progress_clamps_out_of_range_values() {
        let a = number_set("S", &[10.0]);
        let b = number_set("S", &[30.0]);
        // Negative → prev, >1 → next.
        assert_eq!(nth_y(&interpolate_data_set(&a, &b, -1.0), 0, 0), 10.0);
        assert_eq!(nth_y(&interpolate_data_set(&a, &b, 5.0), 0, 0), 30.0);
    }

    #[test]
    fn x_values_come_from_next() {
        // Swap X from numeric indices to strings — ensure the blended
        // set carries the new X so axis labels don't flicker.
        let a = ChartDataSet::single(ChartDataSeries::from_points(
            "S",
            vec![ChartPoint::new(0, 10.0), ChartPoint::new(1, 20.0)],
        ));
        let b = ChartDataSet::single(ChartDataSeries::from_points(
            "S",
            vec![ChartPoint::new("Mon", 30.0), ChartPoint::new("Tue", 40.0)],
        ));
        let blended = interpolate_data_set(&a, &b, 0.4);
        match &blended.series[0].inner.points[0].x {
            PlottableValue::Category(s) => assert_eq!(s.as_ref(), "Mon"),
            other => panic!("expected next X (Category \"Mon\"), got {other:?}"),
        }
    }

    #[test]
    fn non_numeric_y_crossfades_at_midpoint() {
        let t0 = SystemTime::UNIX_EPOCH;
        let t1 = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(60 * 60 * 24);
        let a = ChartDataSet::single(ChartDataSeries::from_points(
            "S",
            vec![ChartPoint::new(0, PlottableValue::Date(t0))],
        ));
        let b = ChartDataSet::single(ChartDataSeries::from_points(
            "S",
            vec![ChartPoint::new(0, PlottableValue::Date(t1))],
        ));

        // Before midpoint → prev.
        let early = interpolate_data_set(&a, &b, 0.3);
        assert_eq!(early.series[0].inner.points[0].y, PlottableValue::Date(t0));
        // After midpoint → next.
        let late = interpolate_data_set(&a, &b, 0.7);
        assert_eq!(late.series[0].inner.points[0].y, PlottableValue::Date(t1));
    }

    #[test]
    fn ragged_point_counts_crossfade_trailing_points() {
        let a = number_set("S", &[10.0, 20.0]);
        let b = number_set("S", &[30.0, 40.0, 50.0]);

        // Before midpoint: extra trailing point from `next` not yet present.
        let early = interpolate_data_set(&a, &b, 0.3);
        assert_eq!(early.series[0].inner.points.len(), 2);

        // After midpoint: extra point appears.
        let late = interpolate_data_set(&a, &b, 0.7);
        assert_eq!(late.series[0].inner.points.len(), 3);
        assert_eq!(nth_y(&late, 0, 2), 50.0);
    }

    #[test]
    fn ragged_series_counts_crossfade_extra_series() {
        let a = ChartDataSet::multi(vec![ChartSeries::new(ChartDataSeries::new(
            "A",
            vec![10.0, 20.0],
        ))]);
        let b = ChartDataSet::multi(vec![
            ChartSeries::new(ChartDataSeries::new("A", vec![30.0, 40.0])),
            ChartSeries::new(ChartDataSeries::new("B", vec![1.0, 2.0])),
        ]);

        let early = interpolate_data_set(&a, &b, 0.3);
        assert_eq!(early.series.len(), 1);

        let late = interpolate_data_set(&a, &b, 0.7);
        assert_eq!(late.series.len(), 2);
        assert_eq!(late.series[1].inner.name.as_ref(), "B");
    }

    #[test]
    fn range_high_bound_lerps_alongside_low() {
        let a = ChartDataSet::single(ChartDataSeries::range(
            "Band",
            vec![0.0, 10.0],
            vec![20.0, 30.0],
        ));
        let b = ChartDataSet::single(ChartDataSeries::range(
            "Band",
            vec![40.0, 50.0],
            vec![60.0, 70.0],
        ));
        let blended = interpolate_data_set(&a, &b, 0.5);
        let p0 = &blended.series[0].inner.points[0];
        assert!((p0.y.as_number_f32().unwrap() - 20.0).abs() < 1e-4);
        assert!(
            (p0.y_high.as_ref().unwrap().as_number_f32().unwrap() - 40.0).abs() < 1e-4,
            "y_high should lerp alongside y"
        );
    }

    #[test]
    fn z_channel_lerps_for_rectangle_marks() {
        let a = ChartDataSet::single(ChartDataSeries::from_points(
            "Heat",
            vec![ChartPoint::new(0, 0).with_z(10.0)],
        ));
        let b = ChartDataSet::single(ChartDataSeries::from_points(
            "Heat",
            vec![ChartPoint::new(0, 0).with_z(30.0)],
        ));
        let blended = interpolate_data_set(&a, &b, 0.5);
        let p0 = &blended.series[0].inner.points[0];
        assert!((p0.z.as_ref().unwrap().as_number_f32().unwrap() - 20.0).abs() < 1e-4);
    }
}
