//! Chart data types.

use std::sync::Arc;
use std::time::SystemTime;

use gpui::{Hsla, SharedString};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ChartType
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Chart mark type.
///
/// Mirrors Swift Charts' `Mark` vocabulary. Render backends:
/// - Bar, Point: div-based primitives.
/// - Line, Area, Range, Rule, Sector, Rectangle: canvas paint callbacks.
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
    /// Pie / donut sectors. Each series contributes one slice sized by the
    /// first point's `y` value. Pair with
    /// [`Chart::inner_radius_ratio`](super::render::Chart::inner_radius_ratio)
    /// to turn a pie into a donut.
    Sector,
    /// Heatmap cells. Each point's `x` and `y` select a cell in the grid
    /// and the magnitude comes from the point's `z` channel (fall back to
    /// `y` when `z` is absent).
    Rectangle,
}

/// Orientation of Bar marks.
///
/// Mirrors Swift Charts' default vs. flipped bar layout. `Vertical` is the
/// classic column chart; `Horizontal` pivots the axes so bars grow
/// left-to-right from a Y-aligned baseline and slots distribute
/// top-to-bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BarOrientation {
    /// Bars grow upwards from the baseline; slots run left-to-right.
    #[default]
    Vertical,
    /// Bars grow rightwards from the Y-axis; slots run top-to-bottom.
    Horizontal,
}

/// Legend placement relative to the plot area.
///
/// Mirrors Swift Charts' `.chartLegend(position:)` surface. `Automatic`
/// (default) resolves to the historical behaviour: bottom for multi-
/// series charts, hidden for single-series charts unless
/// [`Chart::show_legend`](super::render::Chart::show_legend)` forces it on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegendPosition {
    /// HIG default: bottom for multi-series, hidden for single-series.
    #[default]
    Automatic,
    /// Legend row above the plot area.
    Top,
    /// Legend row below the plot area.
    Bottom,
    /// Legend column to the left of the plot area.
    Leading,
    /// Legend column to the right of the plot area.
    Trailing,
    /// Suppress the legend regardless of `show_legend`.
    Hidden,
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
            ChartType::Sector => "sector",
            ChartType::Rectangle => "heatmap",
        }
    }

    /// Whether this chart type anchors its y-axis at zero (HIG requirement
    /// for bar charts so relative heights remain comparable).
    pub(crate) fn anchors_at_zero(self) -> bool {
        matches!(self, Self::Bar | Self::Area | Self::Range)
    }

    /// Whether this chart type uses its own plot geometry (polar for
    /// Sector, grid for Rectangle) and therefore should skip the standard
    /// linear axis / gridline overlay pipeline.
    pub(crate) fn uses_custom_plot_geometry(self) -> bool {
        matches!(self, Self::Sector | Self::Rectangle)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// PlottableValue / ChartPoint
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A value that can be placed on a chart axis.
///
/// Mirrors Swift Charts' `Plottable` protocol: a number (continuous),
/// a date (time-series), or a category (discrete ordinal). A [`Scale`]
/// converts these to normalised plot-area coordinates; the categorical
/// colour palette and per-mark painters read them via [`as_number_f32`]
/// when a numeric fallback is useful.
///
/// [`Scale`]: crate::components::content::chart
/// [`as_number_f32`]: PlottableValue::as_number_f32
#[derive(Debug, Clone, PartialEq)]
pub enum PlottableValue {
    /// A continuous numeric value (integers, floats, percentages).
    Number(f64),
    /// A calendar/clock instant. Paired with a `DateScale` to get
    /// locale-aware tick labels and granularity-aware grid stepping.
    Date(SystemTime),
    /// A discrete category label. Paired with a `CategoryScale` that
    /// gives each unique value its own slot.
    Category(SharedString),
}

impl PlottableValue {
    /// Extract the number when this is `Number(_)`, else `None`. Date
    /// and Category variants return `None` — their projection needs
    /// a `Scale` for context.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            PlottableValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Same as [`as_number`] but returns `f32` for convenience in paint
    /// callbacks that already work in pixel-space `f32`.
    ///
    /// [`as_number`]: PlottableValue::as_number
    pub fn as_number_f32(&self) -> Option<f32> {
        self.as_number().map(|n| n as f32)
    }
}

impl From<f32> for PlottableValue {
    fn from(n: f32) -> Self {
        Self::Number(n as f64)
    }
}

impl From<f64> for PlottableValue {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<i32> for PlottableValue {
    fn from(n: i32) -> Self {
        Self::Number(n as f64)
    }
}

impl From<usize> for PlottableValue {
    fn from(n: usize) -> Self {
        Self::Number(n as f64)
    }
}

impl From<SystemTime> for PlottableValue {
    fn from(t: SystemTime) -> Self {
        Self::Date(t)
    }
}

impl From<SharedString> for PlottableValue {
    fn from(s: SharedString) -> Self {
        Self::Category(s)
    }
}

impl From<&str> for PlottableValue {
    fn from(s: &str) -> Self {
        Self::Category(SharedString::from(s.to_string()))
    }
}

/// A single `(x, y)` data point plus optional channels.
///
/// `y_high` carries the upper bound for Range marks (with `y` as the
/// lower bound). `z` carries a magnitude channel for Rectangle/heatmap
/// marks so they can encode a third dimension without a second series.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartPoint {
    /// X position. Index-like by default (`Number(0)`, `Number(1)`, …)
    /// so existing sparkline call sites render identically; a
    /// `CategoryScale` or `DateScale` can take over in Phase 1.
    pub x: PlottableValue,
    /// Y position — the primary value for most mark types. For Range
    /// marks this is the lower bound.
    pub y: PlottableValue,
    /// Upper bound for Range marks. `None` for every other mark type.
    pub y_high: Option<PlottableValue>,
    /// Magnitude channel for Rectangle/heatmap marks.
    pub z: Option<PlottableValue>,
}

impl ChartPoint {
    /// Create a `(x, y)` point with no range or magnitude channel.
    pub fn new(x: impl Into<PlottableValue>, y: impl Into<PlottableValue>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            y_high: None,
            z: None,
        }
    }

    /// Create a Range point: `(x, y_low .. y_high)`.
    pub fn range(
        x: impl Into<PlottableValue>,
        y_low: impl Into<PlottableValue>,
        y_high: impl Into<PlottableValue>,
    ) -> Self {
        Self {
            x: x.into(),
            y: y_low.into(),
            y_high: Some(y_high.into()),
            z: None,
        }
    }

    /// Attach a magnitude channel for Rectangle/heatmap marks.
    pub fn with_z(mut self, z: impl Into<PlottableValue>) -> Self {
        self.z = Some(z.into());
        self
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ChartDataSeries / ChartSeries / ChartDataSet
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A single named data series.
///
/// Points are stored in `Arc<[ChartPoint]>` so the canvas render path
/// can refcount-clone instead of deep-copying the buffer into each
/// paint closure. [`ChartDataSeries::new`] and [`ChartDataSeries::range`]
/// remain the common-case constructors for `Vec<f32>` call sites so the
/// refactor doesn't ripple through simple sparkline code.
#[derive(Debug, Clone)]
pub struct ChartDataSeries {
    /// Display name — shown in legends, FKA labels, and the default
    /// VoiceOver announcement.
    pub name: SharedString,
    /// `(x, y, …)` data points. For Range marks each point carries a
    /// `y_high`; for Rectangle marks each point carries a `z` magnitude.
    pub points: Arc<[ChartPoint]>,
}

impl ChartDataSeries {
    /// Create a single-value series from a `Vec<f32>`.
    ///
    /// Each value `v_i` is paired with its index: `ChartPoint { x:
    /// Number(i), y: Number(v_i) }`. Matches the v2 API so existing
    /// call sites compile unchanged; pair with a [`CategoryScale`] or
    /// [`DateScale`] at the chart level to drive X from real values.
    ///
    /// [`CategoryScale`]: crate::components::content::chart
    /// [`DateScale`]: crate::components::content::chart
    pub fn new(name: impl Into<SharedString>, values: Vec<f32>) -> Self {
        let points: Vec<ChartPoint> = values
            .into_iter()
            .enumerate()
            .map(|(i, v)| ChartPoint::new(i, v))
            .collect();
        Self {
            name: name.into(),
            points: points.into(),
        }
    }

    /// Create a Range series from parallel lower- and upper-bound arrays.
    ///
    /// Each point becomes `ChartPoint { y: low[i], y_high: Some(high[i])
    /// }`. Truncates to the shorter array so mismatched inputs don't
    /// panic — mirrors the prior `paint_range` contract.
    pub fn range(name: impl Into<SharedString>, low: Vec<f32>, high: Vec<f32>) -> Self {
        let count = low.len().min(high.len());
        let points: Vec<ChartPoint> = (0..count)
            .map(|i| ChartPoint::range(i, low[i], high[i]))
            .collect();
        Self {
            name: name.into(),
            points: points.into(),
        }
    }

    /// Create a series from explicit [`ChartPoint`]s.
    pub fn from_points(
        name: impl Into<SharedString>,
        points: impl Into<Arc<[ChartPoint]>>,
    ) -> Self {
        Self {
            name: name.into(),
            points: points.into(),
        }
    }

    /// Minimum Y value across the series, honouring Range bounds.
    ///
    /// Non-Number Y values (Date/Category) are skipped — they have no
    /// numeric meaning without a [`Scale`]. Phase 1 will route axis
    /// extent through the scale and retire this helper.
    pub(crate) fn min_value(&self) -> f32 {
        let mut m = f32::INFINITY;
        for p in self.points.iter() {
            if let Some(y) = p.y.as_number_f32() {
                m = m.min(y);
            }
            if let Some(yh) = p.y_high.as_ref().and_then(|v| v.as_number_f32()) {
                m = m.min(yh);
            }
        }
        m
    }

    /// Maximum Y value across the series, honouring Range bounds.
    pub(crate) fn max_value(&self) -> f32 {
        let mut m = f32::NEG_INFINITY;
        for p in self.points.iter() {
            if let Some(y) = p.y.as_number_f32() {
                m = m.max(y);
            }
            if let Some(yh) = p.y_high.as_ref().and_then(|v| v.as_number_f32()) {
                m = m.max(yh);
            }
        }
        m
    }
}

/// A series with an optional per-series colour override.
#[derive(Debug, Clone)]
pub struct ChartSeries {
    /// The underlying data series.
    pub inner: ChartDataSeries,
    /// Override the auto-assigned palette colour. Takes priority over the
    /// global `Chart::color(…)` for this series.
    pub color: Option<Hsla>,
}

impl ChartSeries {
    /// Wrap a [`ChartDataSeries`] with no colour override (palette-assigned).
    pub fn new(series: ChartDataSeries) -> Self {
        Self {
            inner: series,
            color: None,
        }
    }

    /// Override the series colour.
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
    /// The series rendered on the chart, in legend order.
    pub series: Vec<ChartSeries>,
}

impl ChartDataSet {
    /// Create a single-series dataset.
    pub fn single(series: ChartDataSeries) -> Self {
        Self {
            series: vec![ChartSeries::new(series)],
        }
    }

    /// Create a multi-series dataset.
    pub fn multi(series: Vec<ChartSeries>) -> Self {
        Self { series }
    }

    /// Whether this data set contains multiple series.
    pub fn is_multi(&self) -> bool {
        self.series.len() > 1
    }

    /// The longest series length — drives slot width and FKA focus count.
    pub(crate) fn max_points(&self) -> usize {
        self.series
            .iter()
            .map(|s| s.inner.points.len())
            .max()
            .unwrap_or(0)
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

/// Width of a single bar given the slot width and number of series per slot.
///
/// Floor at 1 px so a very-dense multi-series chart still draws a visible
/// mark at every slot. Shared between `render.rs` and unit tests so a change
/// to the formula is picked up in both places.
pub(crate) fn bar_width(slot_width: f32, n_series: usize) -> f32 {
    let count = n_series.max(1) as f32;
    let total_gap = BAR_GAP * (count - 1.0);
    ((slot_width * BAR_WIDTH_RATIO - total_gap) / count).max(1.0)
}

/// Diameter of a point marker given the slot width.
///
/// Clamped to `[MIN_POINT_SIZE, MAX_POINT_SIZE]` so points stay visible in
/// dense charts and readable in sparse ones.
pub(crate) fn point_size(slot_width: f32) -> f32 {
    MIN_POINT_SIZE.max(slot_width.min(MAX_POINT_SIZE))
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// AxisPosition / AxisTickStyle / GridLineStyle / AxisMarks
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Where an axis renders relative to the plot area.
///
/// Mirrors Swift Charts' `AxisMarkPosition`. `Automatic` picks the HIG
/// default per axis — leading Y and bottom X, matching what most
/// dashboards show today. `Leading` / `Trailing` only apply to the Y
/// axis; `Top` / `Bottom` only apply to the X axis. A mismatched value
/// (e.g. `Leading` on an X axis) is treated as `Automatic`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AxisPosition {
    /// HIG default: Leading Y, Bottom X.
    #[default]
    Automatic,
    /// Y axis on the leading (left) edge.
    Leading,
    /// Y axis on the trailing (right) edge — Swift Charts' common pairing
    /// with a leading-aligned layout so the chart's left edge aligns with
    /// surrounding interface elements.
    Trailing,
    /// X axis on the top edge.
    Top,
    /// X axis on the bottom edge.
    Bottom,
}

/// How tick marks and value labels are generated.
///
/// Mirrors Swift Charts' `AxisMarkValues`. `Manual` supplies explicit
/// tick positions; `Hidden` suppresses both the tick mark and the value
/// label while leaving gridlines controlled separately by
/// [`GridLineStyle`].
#[derive(Debug, Clone, PartialEq, Default)]
pub enum AxisTickStyle {
    /// Tick count + `nice_ticks` rounding (the historical default).
    #[default]
    Automatic,
    /// Explicit tick positions. Numeric values only — Date/Category
    /// ticks come from the scale itself.
    Manual(Vec<f32>),
    /// Suppress the tick marks and value labels for this axis.
    Hidden,
}

/// How gridlines render at tick positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridLineStyle {
    /// Continuous 1px line across the plot area (current default).
    #[default]
    Solid,
    /// Dashed line — hand-rolled via short `move_to`/`line_to` pairs so
    /// the effect works through GPUI's `PathBuilder` without a native
    /// line-dash primitive.
    Dashed,
    /// Suppress gridlines without touching the tick labels themselves.
    Hidden,
}

/// Format callback converting a tick value into a display label.
///
/// Wrapped in `Arc<dyn Fn + Send + Sync>` so `AxisConfig` remains
/// `Clone` — every chart re-render clones the axis config into the paint
/// closure, and a naked closure type would make the field un-Clone.
pub type AxisValueFormatter = Arc<dyn Fn(&PlottableValue) -> SharedString + Send + Sync + 'static>;

/// Unified axis configuration knob. One [`AxisConfig`] owns two of these:
/// one for the X axis and one for the Y axis.
///
/// Mirrors Swift Charts' `AxisMarks` builder — position, tick style,
/// gridline style, and an optional value label formatter. Defaults match
/// the HIG: automatic placement, automatic ticks, solid gridlines, and
/// the legacy format (`0`, `42`, `1.5`) from
/// [`super::render::format_y_tick`].
#[derive(Clone, Default)]
pub struct AxisMarks {
    /// Where the axis renders relative to the plot area.
    pub position: AxisPosition,
    /// How tick positions and labels are generated.
    pub tick_style: AxisTickStyle,
    /// How the gridline at each tick is drawn.
    pub grid_line_style: GridLineStyle,
    /// Override the default tick label formatter. Receives the
    /// [`PlottableValue`] at each tick position and returns the visible
    /// label. `None` falls back to the default numeric formatter.
    pub value_label_formatter: Option<AxisValueFormatter>,
}

impl AxisMarks {
    /// Create a default axis-marks configuration (automatic on every knob).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the axis position.
    pub fn position(mut self, position: AxisPosition) -> Self {
        self.position = position;
        self
    }

    /// Set the tick style. See [`AxisTickStyle`] for the vocabulary.
    pub fn tick_style(mut self, style: AxisTickStyle) -> Self {
        self.tick_style = style;
        self
    }

    /// Set the gridline style.
    pub fn grid_line_style(mut self, style: GridLineStyle) -> Self {
        self.grid_line_style = style;
        self
    }

    /// Override the tick label formatter. Pass `|v| v.as_number().map(...)`
    /// (or similar) to produce custom strings like `$42` or `1.2K`.
    pub fn value_label_formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(&PlottableValue) -> SharedString + Send + Sync + 'static,
    {
        self.value_label_formatter = Some(Arc::new(formatter));
        self
    }
}

// Axis marks can't derive Debug because `AxisValueFormatter` is a
// trait object. Emit a stable summary that includes the other knobs
// plus "formatter: Some/None" so chart tests can still assert on it.
impl std::fmt::Debug for AxisMarks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AxisMarks")
            .field("position", &self.position)
            .field("tick_style", &self.tick_style)
            .field("grid_line_style", &self.grid_line_style)
            .field(
                "value_label_formatter",
                &self.value_label_formatter.as_ref().map(|_| "<fn>"),
            )
            .finish()
    }
}

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
    /// Category labels for the X-axis (one per data point). `Arc<[_]>` so
    /// `AxisConfig::clone` only bumps a refcount — charts that redraw on
    /// every hover-frame (see `ChartView`) don't re-allocate the Vec.
    pub x_labels: Option<Arc<[SharedString]>>,
    /// Show the Y-axis line.
    pub show_y_line: bool,
    /// Show the X-axis baseline.
    pub x_baseline: bool,
    /// Override the auto-inferred X-axis scale (Linear/Log/Category/Date).
    /// `None` means the chart infers a scale from the data's
    /// [`PlottableValue`] variant.
    pub x_scale: Option<Arc<dyn super::scales::Scale>>,
    /// Override the auto-inferred Y-axis scale.
    pub y_scale: Option<Arc<dyn super::scales::Scale>>,
    /// Unified Y-axis behaviour (position, tick style, gridline style,
    /// label formatter). Defaults to `AxisMarks::default()` which keeps
    /// the leading Y-axis + solid gridlines + numeric labels of prior
    /// releases.
    pub y_marks: AxisMarks,
    /// Unified X-axis behaviour. Same defaults as `y_marks`.
    pub x_marks: AxisMarks,
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            y_tick_count: 5,
            y_ticks: None,
            x_labels: None,
            show_y_line: false,
            x_baseline: false,
            x_scale: None,
            y_scale: None,
            y_marks: AxisMarks::default(),
            x_marks: AxisMarks::default(),
        }
    }
}

impl AxisConfig {
    /// Create a default axis configuration (5 Y-ticks, no X labels).
    pub fn new() -> Self {
        Self::default()
    }

    /// Approximate Y-axis tick count for the "nice numbers" rounding.
    pub fn y_tick_count(mut self, count: usize) -> Self {
        self.y_tick_count = count;
        self
    }

    /// Override the auto-computed Y-axis ticks with explicit values.
    pub fn y_ticks(mut self, ticks: Vec<f32>) -> Self {
        self.y_ticks = Some(ticks);
        self
    }

    /// Supply one X-axis category label per data point.
    pub fn x_labels(mut self, labels: Vec<impl Into<SharedString>>) -> Self {
        self.x_labels = Some(labels.into_iter().map(|l| l.into()).collect());
        self
    }

    /// Y-axis label column width, derived from the platform's Mini control
    /// tier so callers get consistent label gutters across platforms without
    /// hardcoding a pt value. `control_height(Mini)` is ~16pt on macOS and
    /// ~24pt on touch platforms; doubling lets `~5` Caption1 digits render
    /// right-aligned with a small gutter to the plot area.
    pub(crate) fn y_label_width(theme: &crate::foundations::theme::TahoeTheme) -> f32 {
        theme.control_height(crate::foundations::layout::ControlSize::Mini) * 2.5
    }

    /// X-axis label row height, derived from Caption1's line-height on the
    /// active platform. Using `control_height(Mini)` keeps the row tall
    /// enough to avoid clipping descenders across Dynamic Type scales.
    pub(crate) fn x_label_height(theme: &crate::foundations::theme::TahoeTheme) -> f32 {
        theme.control_height(crate::foundations::layout::ControlSize::Mini) * 1.25
    }

    /// Draw a thin line along the Y-axis at the plot area's left edge.
    pub fn show_y_line(mut self) -> Self {
        self.show_y_line = true;
        self
    }

    /// Draw the X-axis baseline along the bottom of the plot area.
    pub fn x_baseline(mut self) -> Self {
        self.x_baseline = true;
        self
    }

    /// Override the X-axis scale. Accepts any implementation of [`Scale`]
    /// (typically [`LinearScale`], [`LogScale`], [`CategoryScale`], or
    /// [`DateScale`]).
    ///
    /// [`Scale`]: crate::components::content::chart::Scale
    /// [`LinearScale`]: crate::components::content::chart::LinearScale
    /// [`LogScale`]: crate::components::content::chart::LogScale
    /// [`CategoryScale`]: crate::components::content::chart::CategoryScale
    /// [`DateScale`]: crate::components::content::chart::DateScale
    pub fn x_scale<S: super::scales::Scale>(mut self, scale: S) -> Self {
        self.x_scale = Some(Arc::new(scale));
        self
    }

    /// Override the Y-axis scale. See [`x_scale`] for the scale vocabulary.
    ///
    /// [`x_scale`]: AxisConfig::x_scale
    pub fn y_scale<S: super::scales::Scale>(mut self, scale: S) -> Self {
        self.y_scale = Some(Arc::new(scale));
        self
    }

    /// Replace the full Y-axis marks configuration in one call.
    pub fn y_marks(mut self, marks: AxisMarks) -> Self {
        self.y_marks = marks;
        self
    }

    /// Replace the full X-axis marks configuration in one call.
    pub fn x_marks(mut self, marks: AxisMarks) -> Self {
        self.x_marks = marks;
        self
    }

    /// Place the Y-axis on the leading (left) or trailing (right) edge.
    ///
    /// `Automatic` (the default) matches the HIG — leading Y. Use
    /// `Trailing` to align the chart's leading edge with surrounding
    /// interface elements.
    pub fn y_position(mut self, position: AxisPosition) -> Self {
        self.y_marks.position = position;
        self
    }

    /// Place the X-axis labels on the top or bottom edge.
    pub fn x_position(mut self, position: AxisPosition) -> Self {
        self.x_marks.position = position;
        self
    }

    /// Choose the Y-axis tick generator. `Automatic` uses
    /// [`nice_ticks`], `Manual(Vec<f32>)` pins explicit positions, and
    /// `Hidden` suppresses both the tick labels and the gutter reserved
    /// for them.
    pub fn y_tick_style(mut self, style: AxisTickStyle) -> Self {
        self.y_marks.tick_style = style.clone();
        // Keep the legacy `y_ticks` field in sync so existing call-sites
        // that read it directly (tests, subclassed builders) observe the
        // manual override too.
        self.y_ticks = match style {
            AxisTickStyle::Manual(ref v) => Some(v.clone()),
            _ => self.y_ticks,
        };
        self
    }

    /// Choose the X-axis tick generator. Same vocabulary as
    /// [`y_tick_style`](Self::y_tick_style).
    pub fn x_tick_style(mut self, style: AxisTickStyle) -> Self {
        self.x_marks.tick_style = style;
        self
    }

    /// Override the Y-axis gridline style (`Solid` / `Dashed` / `Hidden`).
    pub fn y_grid_line_style(mut self, style: GridLineStyle) -> Self {
        self.y_marks.grid_line_style = style;
        self
    }

    /// Override the X-axis gridline style.
    pub fn x_grid_line_style(mut self, style: GridLineStyle) -> Self {
        self.x_marks.grid_line_style = style;
        self
    }

    /// Install a custom Y-axis label formatter. The closure receives the
    /// [`PlottableValue`] at each tick position.
    pub fn y_value_label_formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(&PlottableValue) -> SharedString + Send + Sync + 'static,
    {
        self.y_marks.value_label_formatter = Some(Arc::new(formatter));
        self
    }

    /// Install a custom X-axis label formatter.
    pub fn x_value_label_formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(&PlottableValue) -> SharedString + Send + Sync + 'static,
    {
        self.x_marks.value_label_formatter = Some(Arc::new(formatter));
        self
    }

    /// Resolve the Y-axis `AxisPosition::Automatic` to the HIG default
    /// (`Leading`). Any other value passes through unchanged.
    pub(crate) fn effective_y_position(&self) -> AxisPosition {
        match self.y_marks.position {
            AxisPosition::Automatic | AxisPosition::Top | AxisPosition::Bottom => {
                AxisPosition::Leading
            }
            other => other,
        }
    }

    /// Resolve the X-axis `AxisPosition::Automatic` to the HIG default
    /// (`Bottom`).
    pub(crate) fn effective_x_position(&self) -> AxisPosition {
        match self.x_marks.position {
            AxisPosition::Automatic | AxisPosition::Leading | AxisPosition::Trailing => {
                AxisPosition::Bottom
            }
            other => other,
        }
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
        // Priority: explicit `AxisTickStyle::Manual` > legacy `y_ticks` >
        // automatic `nice_ticks`. `Hidden` collapses to an empty vec so
        // the label column is not allocated.
        match &self.y_marks.tick_style {
            AxisTickStyle::Manual(values) => return values.clone(),
            AxisTickStyle::Hidden => return Vec::new(),
            AxisTickStyle::Automatic => {}
        }
        if let Some(ref ticks) = self.y_ticks {
            return ticks.clone();
        }
        if self.y_tick_count == 0 {
            return Vec::new();
        }
        nice_ticks(min, max, self.y_tick_count)
    }
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
    // Rounding up past `f32::MAX` produces infinity, which would then
    // overflow the tick loop and emit non-finite ticks. Fall back to
    // the raw bounds when the rounded range escapes the finite axis.
    if !nice_min.is_finite() || !nice_max.is_finite() {
        return vec![lo];
    }
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
        if !v.is_finite() {
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
    /// Gridline stroke style. Defaults to [`GridLineStyle::Solid`].
    pub style: GridLineStyle,
}

impl GridlineConfig {
    /// Create a configuration with no gridlines (same as `default()`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Horizontal gridlines only.
    pub fn horizontal() -> Self {
        Self {
            horizontal: true,
            ..Self::default()
        }
    }

    /// Vertical gridlines only.
    pub fn vertical() -> Self {
        Self {
            vertical: true,
            ..Self::default()
        }
    }

    /// Override the gridline colour.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the gridline stroke style (solid, dashed, or hidden).
    pub fn style(mut self, style: GridLineStyle) -> Self {
        self.style = style;
        self
    }

    /// Whether any gridlines are enabled.
    pub fn is_active(&self) -> bool {
        (self.horizontal || self.vertical) && !matches!(self.style, GridLineStyle::Hidden)
    }
}
