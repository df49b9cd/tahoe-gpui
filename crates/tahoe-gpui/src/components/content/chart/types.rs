//! Chart data types.

use gpui::{Hsla, SharedString};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ChartType
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Chart mark type.
///
/// Mirrors Swift Charts' `Mark` vocabulary. All six mark types render
/// natively via GPUI canvas (Line, Area, Range, Rule) or div-based
/// primitives (Bar, Point).
///
/// `voice_label()` returns the static lowercase mark-type name so the
/// VoiceOver announcement is always honest about the caller's intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ChartType {
    /// Native bar columns. Full HIG coverage.
    #[default]
    Bar,
    /// Canvas-stroked polyline connecting data points.
    Line,
    /// Canvas-filled area under a polyline.
    Area,
    /// Point sparkline. Full HIG coverage (no stroke needed).
    Point,
    /// Canvas-filled band between lower/upper values.
    Range,
    /// Canvas-stroked horizontal reference line. Only the first value of
    /// the series is drawn (mirrors Swift Charts' `RuleMark(y:)`); extra
    /// values are ignored. In debug builds a `debug_assert!` surfaces calls
    /// that pass more than one value so the misuse is caught in tests.
    Rule,
}

impl ChartType {
    /// The static lowercase mark-type name used for VoiceOver announcements
    /// ("bar chart: …", "line chart: …").
    pub fn voice_label(self) -> &'static str {
        match self {
            ChartType::Bar => "bar",
            ChartType::Line => "line",
            ChartType::Area => "area",
            ChartType::Point => "point",
            ChartType::Range => "range",
            ChartType::Rule => "rule",
        }
    }

    /// Whether this chart type anchors its y-axis at zero (HIG requirement
    /// for bar charts so relative heights remain comparable).
    pub(crate) fn anchors_at_zero(self) -> bool {
        matches!(self, Self::Bar | Self::Area | Self::Range)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ChartDataSeries / ChartSeries / ChartDataSet
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A single named data series.
#[derive(Debug, Clone)]
pub struct ChartDataSeries {
    pub name: SharedString,
    pub values: Vec<f32>,
    /// Lower-bound values for Range charts. When `None`, the series is
    /// treated as a simple value series (Bar, Line, Area, Point, Rule).
    pub range_low: Option<Vec<f32>>,
}

impl ChartDataSeries {
    pub fn new(name: impl Into<SharedString>, values: Vec<f32>) -> Self {
        Self {
            name: name.into(),
            values,
            range_low: None,
        }
    }

    /// Create a Range series with separate lower and upper bound arrays.
    ///
    /// `values` is the upper bound, `low` is the lower bound.
    pub fn range(name: impl Into<SharedString>, low: Vec<f32>, high: Vec<f32>) -> Self {
        Self {
            name: name.into(),
            values: high,
            range_low: Some(low),
        }
    }

    pub(crate) fn min_value(&self) -> f32 {
        let v_min = self.values.iter().copied().fold(f32::INFINITY, f32::min);
        match &self.range_low {
            Some(low) => low.iter().copied().fold(v_min, f32::min),
            None => v_min,
        }
    }

    pub(crate) fn max_value(&self) -> f32 {
        self.values
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max)
    }
}

/// A series with an optional per-series colour override.
#[derive(Debug, Clone)]
pub struct ChartSeries {
    pub inner: ChartDataSeries,
    pub color: Option<Hsla>,
}

impl ChartSeries {
    pub fn new(series: ChartDataSeries) -> Self {
        Self {
            inner: series,
            color: None,
        }
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

/// A collection of series rendered together on one chart.
///
/// When more than one series is present, colours are auto-assigned from
/// the theme's categorical palette and a legend is shown automatically.
#[derive(Debug, Clone)]
pub struct ChartDataSet {
    pub series: Vec<ChartSeries>,
}

impl ChartDataSet {
    pub fn single(series: ChartDataSeries) -> Self {
        Self {
            series: vec![ChartSeries::new(series)],
        }
    }

    pub fn multi(series: Vec<ChartSeries>) -> Self {
        Self { series }
    }

    /// Whether this data set contains multiple series.
    pub fn is_multi(&self) -> bool {
        self.series.len() > 1
    }

    /// Global min across all series.
    pub(crate) fn global_min(&self) -> f32 {
        self.series
            .iter()
            .map(|s| s.inner.min_value())
            .fold(f32::INFINITY, f32::min)
    }

    /// Global max across all series.
    pub(crate) fn global_max(&self) -> f32 {
        self.series
            .iter()
            .map(|s| s.inner.max_value())
            .fold(f32::NEG_INFINITY, f32::max)
    }
}

impl From<ChartDataSeries> for ChartDataSet {
    fn from(series: ChartDataSeries) -> Self {
        Self::single(series)
    }
}

/// Ratio of bar width to slot width.
pub(crate) const BAR_WIDTH_RATIO: f32 = 0.7;
/// Minimum point-marker diameter for sparkline marks.
pub(crate) const MIN_POINT_SIZE: f32 = 4.0;
/// Maximum point-marker diameter (caps the growth from a large slot width).
pub(crate) const MAX_POINT_SIZE: f32 = 10.0;
/// Horizontal gap between bars inside a multi-series slot.
pub(crate) const BAR_GAP: f32 = 1.0;
/// Vertical gap between chart title/subtitle and the plot area.
pub(crate) const TITLE_GAP: f32 = 4.0;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// AxisConfig
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Y-axis and X-axis configuration.
///
/// When `None` (the default), the chart renders at full size with no
/// axis labels or margins — the v1 sparkline mode. When present, margins
/// are allocated for axis labels and tick values are rendered alongside
/// the plot area.
#[derive(Debug, Clone)]
pub struct AxisConfig {
    /// Approximate number of Y-axis ticks. Defaults to 5.
    pub y_tick_count: usize,
    /// Override tick positions with explicit values.
    pub y_ticks: Option<Vec<f32>>,
    /// Category labels for the X-axis (one per data point).
    pub x_labels: Option<Vec<SharedString>>,
    /// Show the Y-axis line.
    pub show_y_line: bool,
    /// Show the X-axis baseline.
    pub x_baseline: bool,
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            y_tick_count: 5,
            y_ticks: None,
            x_labels: None,
            show_y_line: false,
            x_baseline: false,
        }
    }
}

impl AxisConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn y_tick_count(mut self, count: usize) -> Self {
        self.y_tick_count = count;
        self
    }

    pub fn y_ticks(mut self, ticks: Vec<f32>) -> Self {
        self.y_ticks = Some(ticks);
        self
    }

    pub fn x_labels(mut self, labels: Vec<impl Into<SharedString>>) -> Self {
        self.x_labels = Some(labels.into_iter().map(|l| l.into()).collect());
        self
    }

    pub fn show_y_line(mut self) -> Self {
        self.show_y_line = true;
        self
    }

    pub fn x_baseline(mut self) -> Self {
        self.x_baseline = true;
        self
    }

    /// Whether any axis rendering is needed.
    pub fn is_active(&self) -> bool {
        self.y_ticks.is_some()
            || self.y_tick_count > 0
            || self.x_labels.is_some()
            || self.show_y_line
            || self.x_baseline
    }

    /// Compute Y-axis tick values using "nice numbers" algorithm.
    pub(crate) fn compute_y_ticks(&self, min: f32, max: f32) -> Vec<f32> {
        if let Some(ref ticks) = self.y_ticks {
            return ticks.clone();
        }
        if self.y_tick_count == 0 {
            return Vec::new();
        }
        nice_ticks(min, max, self.y_tick_count)
    }

    /// Y-axis label column width in pixels.
    pub(crate) const Y_LABEL_WIDTH: f32 = 40.0;
    /// X-axis label row height in pixels.
    pub(crate) const X_LABEL_HEIGHT: f32 = 20.0;
}

/// Compute "nice" tick values for a range.
///
/// Rounds to 1, 2, or 5 multiples of powers of 10 so axes show clean
/// values like 0, 20, 40, 60 instead of 0, 17.3, 34.6, …
///
/// Degenerate inputs (`NaN`, infinities, `min > max`, zero-width ranges,
/// zero count) return a singleton `[min]` rather than looping — the caller
/// is responsible for drawing a single-tick axis, not this function.
pub(crate) fn nice_ticks(min: f32, max: f32, count: usize) -> Vec<f32> {
    if count == 0 || !min.is_finite() || !max.is_finite() {
        return vec![if min.is_finite() { min } else { 0.0 }];
    }
    let (lo, hi) = if max < min { (max, min) } else { (min, max) };
    if (hi - lo).abs() < f32::EPSILON {
        return vec![lo];
    }

    let range = hi - lo;
    let rough_step = range / count.max(1) as f32;
    let mag = 10_f32.powf(rough_step.log10().floor());
    let nice_step = if rough_step / mag < 1.5 {
        mag
    } else if rough_step / mag < 3.0 {
        2.0 * mag
    } else if rough_step / mag < 7.0 {
        5.0 * mag
    } else {
        10.0 * mag
    };

    // Guard: log10 / powf round-trips can produce non-finite or zero
    // steps for extreme inputs. Without this the `while` loop below would
    // either spin forever (step == 0) or never advance (step NaN).
    if !nice_step.is_finite() || nice_step <= 0.0 {
        return vec![lo];
    }

    let nice_min = (lo / nice_step).floor() * nice_step;
    let nice_max = (hi / nice_step).ceil() * nice_step;
    let stop = nice_max + nice_step * 0.01;

    let mut ticks = Vec::new();
    let mut v = nice_min;
    // Hard cap on iterations — defends against pathological floating-point
    // cases where `v += nice_step` stalls due to catastrophic cancellation.
    let max_iters = count.saturating_mul(4).max(32);
    for _ in 0..max_iters {
        if v > stop {
            break;
        }
        ticks.push(v);
        v += nice_step;
    }
    ticks
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// GridlineConfig
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Gridline configuration.
///
/// Painted at Y-coordinates matching axis ticks via canvas, behind data
/// marks. Default: no gridlines (backward compatible).
#[derive(Debug, Clone, Default)]
pub struct GridlineConfig {
    /// Show horizontal gridlines at Y-axis tick positions.
    pub horizontal: bool,
    /// Show vertical gridlines at each data point.
    pub vertical: bool,
    /// Override colour. Defaults to `theme.separator_color()`.
    pub color: Option<Hsla>,
}

impl GridlineConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn horizontal() -> Self {
        Self {
            horizontal: true,
            ..Self::default()
        }
    }

    pub fn vertical() -> Self {
        Self {
            vertical: true,
            ..Self::default()
        }
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    pub fn is_active(&self) -> bool {
        self.horizontal || self.vertical
    }
}
