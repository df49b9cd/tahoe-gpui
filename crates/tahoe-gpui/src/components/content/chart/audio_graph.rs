//! Audio Graphs accessibility for [`Chart`] and [`ChartView`].
//!
//! Mirrors Apple's [`AXChartDescriptor`] surface — a structured description
//! of a chart's axes and data that assistive technology can turn into an
//! audible trace. VoiceOver's `VO + Shift + S` shortcut asks a chart to
//! sonify itself, pitching each data point against its Y value and pacing
//! the tones to the inter-point X spacing.
//!
//! GPUI `v0.231.1-pre` does not expose an audio output primitive, and
//! the crate's `voice` feature is scoped to microphone capture rather than
//! speaker playback. Until GPUI ships a speaker API, sonification emits a
//! structured tone sequence (see [`ChartDescriptor::tone_sequence`]) and
//! [`ChartDescriptor::play_sonification`] logs once in debug builds that
//! audio would have played. When VoiceOver is unavailable, the whole API
//! is a no-op — no runtime cost for hearing users. Hosts that want real
//! audio can delegate to AVFoundation today by calling `tone_sequence`
//! and feeding the `(frequency, duration)` pairs to `AVAudioEngine`.
//!
//! # HIG
//!
//! Apple's HIG *Charting data* page explicitly calls out Audio Graphs:
//! *"Consider Audio Graphs for VoiceOver users — an audio representation
//! of your chart that people can listen to as well as read."* This module
//! is the wiring point.
//!
//! [`Chart`]: super::render::Chart
//! [`ChartView`]: super::view::ChartView
//! [`AXChartDescriptor`]: https://developer.apple.com/documentation/accessibility/axchartdescriptor

use gpui::SharedString;

use super::types::{ChartDataSet, PlottableValue};

/// Lowest and highest sonification pitch in Hz. Covers roughly D3–D6 (the
/// range Apple's Audio Graphs use so the tones stay musical across typical
/// chart scales). Callers can override via [`ChartDescriptor::pitch_range`].
const DEFAULT_PITCH_LOW_HZ: f32 = 146.83;
const DEFAULT_PITCH_HIGH_HZ: f32 = 1_174.66;

/// Default sonification duration per data point (milliseconds). Apple's
/// Audio Graphs default is around 300 ms per point; we use 200 ms so a
/// 30-point chart reads in ~6 seconds rather than ~9.
const DEFAULT_TONE_DURATION_MS: f32 = 200.0;

/// Describes one axis of a chart for assistive-tech consumption.
///
/// Mirrors `AXDataAxisDescriptor` — the title is what VoiceOver reads to
/// introduce the axis, the range describes the span that non-visual tools
/// will scale values across (e.g. pitch mapping for Audio Graphs), and
/// the unit suffix lets the descriptor read as "25 degrees" rather than
/// bare "25".
#[derive(Debug, Clone, PartialEq)]
pub struct AxisDescriptor {
    /// Human-readable axis title ("Temperature", "Quarter", "Sales").
    pub title: SharedString,
    /// Inclusive numeric extent of the axis, used by the sonifier to map
    /// values to pitch/time. Non-numeric axes (dates, categories) should
    /// pass `(0.0, n as f64)` where `n` is the point count.
    pub range: (f64, f64),
    /// Optional unit suffix ("°C", "dollars"). `None` leaves VoiceOver to
    /// read the bare number.
    pub unit: Option<SharedString>,
}

impl AxisDescriptor {
    /// Build an axis descriptor from its title and numeric range.
    pub fn new(title: impl Into<SharedString>, range: (f64, f64)) -> Self {
        Self {
            title: title.into(),
            range,
            unit: None,
        }
    }

    /// Attach a unit suffix so assistive tech reads values with the unit.
    pub fn unit(mut self, unit: impl Into<SharedString>) -> Self {
        self.unit = Some(unit.into());
        self
    }
}

/// Describes one series of a chart for assistive-tech consumption.
///
/// Mirrors `AXDataSeriesDescriptor`. Points are `(x, y)` numeric pairs —
/// non-numeric [`PlottableValue`] variants drop to `None` and are skipped
/// when the series is sonified.
#[derive(Debug, Clone, PartialEq)]
pub struct SeriesDescriptor {
    /// Series name (e.g. "Sales", "Target").
    pub name: SharedString,
    /// Ordered numeric `(x, y)` samples.
    pub points: Vec<(f64, f64)>,
}

impl SeriesDescriptor {
    /// Build a series descriptor from a name and an iterator of numeric
    /// `(x, y)` samples.
    pub fn new(
        name: impl Into<SharedString>,
        points: impl IntoIterator<Item = (f64, f64)>,
    ) -> Self {
        Self {
            name: name.into(),
            points: points.into_iter().collect(),
        }
    }
}

/// Full accessibility description of a chart.
///
/// Mirrors Apple's `AXChartDescriptor`. The `summary` replaces a chart's
/// auto-generated VoiceOver label when present, and the `series` / axis
/// descriptors drive the Audio Graphs sonification pipeline (see
/// [`ChartDescriptor::tone_sequence`]).
#[derive(Debug, Clone, PartialEq)]
pub struct ChartDescriptor {
    /// Chart title ("Q3 Revenue", "Monthly temperature").
    pub title: SharedString,
    /// One- or two-sentence spoken summary VoiceOver reads before the
    /// user starts exploring the chart. Longer than a bare accessibility
    /// label — this is the "describe the chart" field.
    pub summary: SharedString,
    /// Per-series data descriptors. Empty is valid (e.g. a chart that's
    /// waiting on data).
    pub series: Vec<SeriesDescriptor>,
    /// Horizontal axis description.
    pub x_axis: AxisDescriptor,
    /// Vertical axis description.
    pub y_axis: AxisDescriptor,
    /// Sonification pitch range in Hz. Defaults to D3–D6.
    pub pitch_range: (f32, f32),
    /// Sonification duration per data point in milliseconds. Defaults to
    /// 200 ms.
    pub tone_duration_ms: f32,
}

impl ChartDescriptor {
    /// Build a descriptor from its title, summary, and axis pair. Series
    /// default to empty — call [`ChartDescriptor::series`] to populate.
    pub fn new(
        title: impl Into<SharedString>,
        summary: impl Into<SharedString>,
        x_axis: AxisDescriptor,
        y_axis: AxisDescriptor,
    ) -> Self {
        Self {
            title: title.into(),
            summary: summary.into(),
            series: Vec::new(),
            x_axis,
            y_axis,
            pitch_range: (DEFAULT_PITCH_LOW_HZ, DEFAULT_PITCH_HIGH_HZ),
            tone_duration_ms: DEFAULT_TONE_DURATION_MS,
        }
    }

    /// Attach a list of series descriptors.
    pub fn series(mut self, series: Vec<SeriesDescriptor>) -> Self {
        self.series = series;
        self
    }

    /// Override the sonification pitch range. `low` is the pitch at the
    /// Y axis lower bound; `high` is the pitch at the upper bound.
    pub fn pitch_range(mut self, low: f32, high: f32) -> Self {
        self.pitch_range = (low, high);
        self
    }

    /// Override the per-point tone duration in milliseconds.
    pub fn tone_duration_ms(mut self, ms: f32) -> Self {
        self.tone_duration_ms = ms;
        self
    }

    /// Derive a descriptor from a [`ChartDataSet`]. Numeric-only — non-
    /// numeric `PlottableValue` variants are filtered out per series, and
    /// a series with no numeric points becomes an empty descriptor.
    ///
    /// Axis ranges are computed from the union of all series points;
    /// callers that want custom ranges should build the descriptor by
    /// hand.
    pub fn from_data_set(
        title: impl Into<SharedString>,
        summary: impl Into<SharedString>,
        x_axis_title: impl Into<SharedString>,
        y_axis_title: impl Into<SharedString>,
        data_set: &ChartDataSet,
    ) -> Self {
        let mut x_lo = f64::INFINITY;
        let mut x_hi = f64::NEG_INFINITY;
        let mut y_lo = f64::INFINITY;
        let mut y_hi = f64::NEG_INFINITY;
        let mut series: Vec<SeriesDescriptor> = Vec::with_capacity(data_set.series.len());

        for s in data_set.series.iter() {
            let mut points: Vec<(f64, f64)> = Vec::with_capacity(s.inner.points.len());
            for p in s.inner.points.iter() {
                let (Some(xv), Some(yv)) = (p.x.as_number(), p.y.as_number()) else {
                    continue;
                };
                x_lo = x_lo.min(xv);
                x_hi = x_hi.max(xv);
                y_lo = y_lo.min(yv);
                y_hi = y_hi.max(yv);
                points.push((xv, yv));
            }
            series.push(SeriesDescriptor::new(s.inner.name.clone(), points));
        }

        // Fallback for empty / all-non-numeric data so the axis ranges
        // stay finite and the sonifier doesn't divide by NaN.
        if !x_lo.is_finite() {
            x_lo = 0.0;
        }
        if !x_hi.is_finite() || (x_hi - x_lo).abs() < f64::EPSILON {
            x_hi = x_lo + 1.0;
        }
        if !y_lo.is_finite() {
            y_lo = 0.0;
        }
        if !y_hi.is_finite() || (y_hi - y_lo).abs() < f64::EPSILON {
            y_hi = y_lo + 1.0;
        }

        Self {
            title: title.into(),
            summary: summary.into(),
            series,
            x_axis: AxisDescriptor::new(x_axis_title, (x_lo, x_hi)),
            y_axis: AxisDescriptor::new(y_axis_title, (y_lo, y_hi)),
            pitch_range: (DEFAULT_PITCH_LOW_HZ, DEFAULT_PITCH_HIGH_HZ),
            tone_duration_ms: DEFAULT_TONE_DURATION_MS,
        }
    }

    /// Build a sonification tone sequence for the primary (first) series.
    ///
    /// Each tone is `(frequency_hz, duration_ms)`. Frequency maps each
    /// Y value linearly across [`pitch_range`](Self::pitch_range); duration
    /// is [`tone_duration_ms`](Self::tone_duration_ms) per point — inter-
    /// point X spacing is implicit (uniform duration is what Apple's
    /// default sonifier uses). Returns an empty sequence when the series
    /// is empty or the Y axis has zero width.
    pub fn tone_sequence(&self) -> Vec<(f32, f32)> {
        let Some(series) = self.series.first() else {
            return Vec::new();
        };
        let (y_lo, y_hi) = self.y_axis.range;
        let y_span = y_hi - y_lo;
        if y_span.abs() < f64::EPSILON {
            return Vec::new();
        }
        let (p_lo, p_hi) = self.pitch_range;
        let p_span = p_hi - p_lo;
        let duration = self.tone_duration_ms.max(0.0);

        series
            .points
            .iter()
            .map(|(_, y)| {
                let t = (((*y) - y_lo) / y_span).clamp(0.0, 1.0) as f32;
                let freq = p_lo + p_span * t;
                (freq, duration)
            })
            .collect()
    }

    /// Trigger sonification. No-op today pending a GPUI speaker API; logs
    /// once per process in debug builds so the gap surfaces during
    /// development.
    ///
    /// Hosts that want real audio can call [`tone_sequence`](Self::tone_sequence)
    /// directly and drive an `AVAudioEngine` / `cpal` sink themselves.
    pub fn play_sonification(&self) {
        warn_once_sonification_stubbed();
    }
}

/// Emits at most one stderr warning per process when
/// [`ChartDescriptor::play_sonification`] is called without a real audio
/// backend. Keeps the demo surface honest without spamming stderr on
/// every keystroke.
fn warn_once_sonification_stubbed() {
    if cfg!(debug_assertions) && !cfg!(test) {
        use std::sync::atomic::{AtomicBool, Ordering};
        static WARNED: AtomicBool = AtomicBool::new(false);
        if WARNED.swap(true, Ordering::Relaxed) {
            return;
        }
        eprintln!(
            "[tahoe-gpui] ChartDescriptor::play_sonification is a no-op — \
             GPUI v0.231.1-pre has no speaker API, so the tone sequence is \
             computed but not emitted. Call ChartDescriptor::tone_sequence \
             and drive AVAudioEngine yourself for real audio (this warning \
             fires once per process)."
        );
    }
}

/// Resolve a series' Y value at a given index for `PlottableValue` lookups.
/// Used by the descriptor-derivation path to skip non-numeric series.
#[allow(dead_code)]
pub(crate) fn is_numeric(v: &PlottableValue) -> bool {
    v.as_number().is_some()
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::*;
    use crate::components::content::chart::types::{ChartDataSeries, ChartDataSet, ChartSeries};

    #[test]
    fn axis_descriptor_unit_builder_sets_unit() {
        let a = AxisDescriptor::new("Temperature", (0.0, 100.0)).unit("°C");
        assert_eq!(a.title.as_ref(), "Temperature");
        assert_eq!(a.range, (0.0, 100.0));
        assert_eq!(a.unit.as_ref().map(|u| u.as_ref()), Some("°C"));
    }

    #[test]
    fn series_descriptor_collects_points() {
        let s = SeriesDescriptor::new("A", [(0.0, 1.0), (1.0, 2.0), (2.0, 3.0)]);
        assert_eq!(s.name.as_ref(), "A");
        assert_eq!(s.points.len(), 3);
        assert_eq!(s.points[0], (0.0, 1.0));
    }

    #[test]
    fn descriptor_defaults_have_sensible_pitch_and_duration() {
        let d = ChartDescriptor::new(
            "T",
            "S",
            AxisDescriptor::new("X", (0.0, 1.0)),
            AxisDescriptor::new("Y", (0.0, 1.0)),
        );
        assert_eq!(d.pitch_range, (DEFAULT_PITCH_LOW_HZ, DEFAULT_PITCH_HIGH_HZ));
        assert_eq!(d.tone_duration_ms, DEFAULT_TONE_DURATION_MS);
        assert!(d.series.is_empty());
    }

    #[test]
    fn descriptor_builders_override_pitch_and_duration() {
        let d = ChartDescriptor::new(
            "T",
            "S",
            AxisDescriptor::new("X", (0.0, 1.0)),
            AxisDescriptor::new("Y", (0.0, 1.0)),
        )
        .pitch_range(220.0, 880.0)
        .tone_duration_ms(150.0);
        assert_eq!(d.pitch_range, (220.0, 880.0));
        assert_eq!(d.tone_duration_ms, 150.0);
    }

    #[test]
    fn from_data_set_computes_axis_ranges_from_points() {
        let set = ChartDataSet::multi(vec![
            ChartSeries::new(ChartDataSeries::new("A", vec![1.0, 5.0, 3.0])),
            ChartSeries::new(ChartDataSeries::new("B", vec![2.0, 4.0, 6.0])),
        ]);
        let d = ChartDescriptor::from_data_set("Sales", "Quarterly sales", "Quarter", "USD", &set);
        // X axis spans the index range [0, 2].
        assert_eq!(d.x_axis.range, (0.0, 2.0));
        // Y axis spans [1, 6].
        assert_eq!(d.y_axis.range, (1.0, 6.0));
        assert_eq!(d.series.len(), 2);
        assert_eq!(d.series[0].points.len(), 3);
    }

    #[test]
    fn from_data_set_survives_empty_data_set() {
        let set = ChartDataSet::multi(vec![]);
        let d = ChartDescriptor::from_data_set("Empty", "No data", "X", "Y", &set);
        assert!(d.x_axis.range.0.is_finite());
        assert!(d.x_axis.range.1.is_finite());
        assert!(d.y_axis.range.0.is_finite());
        assert!(d.y_axis.range.1.is_finite());
        assert!(d.series.is_empty());
    }

    #[test]
    fn from_data_set_survives_empty_series() {
        let set = ChartDataSet::multi(vec![ChartSeries::new(ChartDataSeries::new("A", vec![]))]);
        let d = ChartDescriptor::from_data_set("T", "S", "X", "Y", &set);
        // One series but no points → ranges must still be finite so tone
        // math never divides by NaN.
        assert!((d.y_axis.range.1 - d.y_axis.range.0).abs() > f64::EPSILON);
        assert_eq!(d.series.len(), 1);
        assert!(d.series[0].points.is_empty());
    }

    #[test]
    fn tone_sequence_maps_y_linearly_across_pitch_range() {
        let d = ChartDescriptor::new(
            "T",
            "S",
            AxisDescriptor::new("X", (0.0, 3.0)),
            AxisDescriptor::new("Y", (0.0, 10.0)),
        )
        .series(vec![SeriesDescriptor::new(
            "A",
            [(0.0, 0.0), (1.0, 5.0), (2.0, 10.0)],
        )])
        .pitch_range(100.0, 200.0);

        let tones = d.tone_sequence();
        assert_eq!(tones.len(), 3);
        // y=0 → low pitch.
        assert!((tones[0].0 - 100.0).abs() < 0.01);
        // y=5 → midpoint pitch.
        assert!((tones[1].0 - 150.0).abs() < 0.01);
        // y=10 → high pitch.
        assert!((tones[2].0 - 200.0).abs() < 0.01);
        // Duration default is 200 ms.
        assert!((tones[0].1 - 200.0).abs() < 0.01);
    }

    #[test]
    fn tone_sequence_clamps_out_of_range_values_to_bounds() {
        // A point above the Y axis range must clamp to the upper pitch
        // rather than extrapolating past it.
        let d = ChartDescriptor::new(
            "T",
            "S",
            AxisDescriptor::new("X", (0.0, 1.0)),
            AxisDescriptor::new("Y", (0.0, 10.0)),
        )
        .series(vec![SeriesDescriptor::new("A", [(0.0, 20.0)])])
        .pitch_range(100.0, 200.0);

        let tones = d.tone_sequence();
        assert_eq!(tones.len(), 1);
        assert!((tones[0].0 - 200.0).abs() < 0.01);
    }

    #[test]
    fn tone_sequence_empty_when_no_series() {
        let d = ChartDescriptor::new(
            "T",
            "S",
            AxisDescriptor::new("X", (0.0, 1.0)),
            AxisDescriptor::new("Y", (0.0, 1.0)),
        );
        assert!(d.tone_sequence().is_empty());
    }

    #[test]
    fn tone_sequence_empty_when_y_span_is_zero() {
        let d = ChartDescriptor::new(
            "T",
            "S",
            AxisDescriptor::new("X", (0.0, 1.0)),
            AxisDescriptor::new("Y", (5.0, 5.0)),
        )
        .series(vec![SeriesDescriptor::new("A", [(0.0, 5.0), (1.0, 5.0)])]);
        // Zero-width Y axis: no meaningful pitch mapping.
        assert!(d.tone_sequence().is_empty());
    }

    #[test]
    fn play_sonification_is_callable() {
        // Smoke-test: play_sonification is a no-op stub today, so calling
        // it must not panic. When a speaker API lands this test should
        // be upgraded to verify audible emission.
        let d = ChartDescriptor::new(
            "T",
            "S",
            AxisDescriptor::new("X", (0.0, 1.0)),
            AxisDescriptor::new("Y", (0.0, 1.0)),
        );
        d.play_sonification();
    }
}
