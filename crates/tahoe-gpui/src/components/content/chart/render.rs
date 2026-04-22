//! Chart component render implementation.

use std::sync::Arc;

use gpui::prelude::*;
use gpui::{
    App, ElementId, FocusHandle, Hsla, Pixels, SharedString, TextAlign, Window, canvas, div, px,
};

use crate::foundations::accessibility::{
    AccessibilityProps, AccessibilityRole, AccessibleExt, FocusGroup,
};
use crate::foundations::icons::Icon;
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};

use super::accessibility::{FkaAttachContext, attach_fka};
use super::annotation::{
    AnnotationContent, AnnotationPosition, ChartAnnotation, resolve_annotation,
};
use super::audio_graph::ChartDescriptor;
use super::interpolation::InterpolationMethod;
use super::marks::{canvas_paint_callback, paint_stacked_area};
use super::rectangle::{build_cells, paint_rectangle_chart};
use super::scales::{LinearScale, Scale};
use super::scroll::ChartScrollConfig;
use super::sector::{DEFAULT_SECTOR_START_ANGLE, paint_sector_chart, sector_weights};
use super::stacking::{MarkStackingMethod, StackSegment, compute_stacks};
use super::types::{
    AxisConfig, AxisPosition, AxisTickStyle, BAR_GAP, BarOrientation, ChartDataSeries,
    ChartDataSet, ChartPoint, ChartSeries, ChartType, GridLineStyle, GridlineConfig,
    LegendPosition, PlottableValue, bar_width, point_size,
};

// BAR_WIDTH_RATIO / MIN_POINT_SIZE / MAX_POINT_SIZE aren't imported at the
// module level — the `bar_width` / `point_size` helpers in `types.rs` fold
// them into their return values so the formulas live in exactly one place.

/// Palette order for auto-assigned multi-series colours.
const PALETTE: &[&str] = &[
    "blue", "green", "orange", "purple", "pink", "teal", "red", "yellow", "cyan", "indigo", "mint",
    "brown",
];

/// Gap in pixels between an annotation and its anchor mark.
const ANNOTATION_GAP_PX: f32 = 6.0;
/// Pixels from the plot edge below which the resolver flips the annotation
/// to the opposite side. Sized roughly for Caption1's line height so the
/// flip kicks in once the caption would otherwise clip.
const ANNOTATION_FLIP_THRESHOLD_PX: f32 = 12.0;
/// Rough width estimate per character for Caption1. Annotations are
/// absolute-positioned and we don't measure layout before painting, so this
/// estimate drives the centring offset for Top/Bottom/Overlay positions.
const ANNOTATION_CHAR_W_PX: f32 = 6.0;
/// Rough line height for Caption1.
const ANNOTATION_LINE_H_PX: f32 = 16.0;
/// Minimum estimated width so very short captions still have a sensible
/// centring box.
const ANNOTATION_MIN_W_PX: f32 = 24.0;

/// HIG chart primitive.
#[derive(IntoElement)]
pub struct Chart {
    pub(crate) id: ElementId,
    pub(crate) chart_type: ChartType,
    pub(crate) data_set: ChartDataSet,
    pub(crate) width: Pixels,
    pub(crate) height: Pixels,
    pub(crate) color: Option<Hsla>,
    pub(crate) accessibility_label: Option<SharedString>,
    pub(crate) point_focus_group: Option<FocusGroup>,
    pub(crate) point_focus_handles: Vec<FocusHandle>,
    pub(crate) axis: Option<AxisConfig>,
    pub(crate) gridlines: Option<GridlineConfig>,
    /// `None` = auto (single-series hides, multi-series shows).
    /// `Some(true)` forces a legend on single-series; `Some(false)` forces
    /// a multi-series chart to hide its legend.
    pub(crate) show_legend: Option<bool>,
    /// Where to place the legend relative to the plot. Defaults to
    /// [`LegendPosition::Automatic`] (bottom for multi-series, hidden for
    /// single-series).
    pub(crate) legend_position: LegendPosition,
    pub(crate) title: Option<SharedString>,
    pub(crate) subtitle: Option<SharedString>,
    /// `[0.0, 0.95]` ratio of sector-chart inner radius to outer radius.
    /// `0.0` = solid pie; `0.6` ≈ donut. Ignored for non-`Sector` charts.
    pub(crate) inner_radius_ratio: f32,
    /// Start angle (radians) for sector charts. Defaults to −π/2 so the
    /// first slice begins at 12 o'clock. Ignored for non-`Sector` charts.
    pub(crate) sector_start_angle: f32,
    /// Mark stacking method for Bar and Area charts. Defaults to
    /// [`MarkStackingMethod::Unstacked`] (series overlay / side-by-side).
    pub(crate) stacking: MarkStackingMethod,
    /// Bar orientation for [`ChartType::Bar`]. Defaults to
    /// [`BarOrientation::Vertical`]; [`BarOrientation::Horizontal`] flips
    /// slots to the Y axis and grows bars left-to-right.
    pub(crate) bar_orientation: BarOrientation,
    /// Annotations pinned to data points or raw values. Rendered as
    /// absolute-positioned overlays inside the plot area.
    pub(crate) annotations: Vec<ChartAnnotation>,
    /// Interpolation used to connect data points for Line, Area, Range,
    /// and stacked-area marks. Defaults to
    /// [`InterpolationMethod::CatmullRom`] so the visual output matches
    /// previous releases.
    pub(crate) interpolation: InterpolationMethod,
    /// Optional scroll / zoom configuration. When set with a narrower
    /// `x_visible_domain` than the underlying data, points outside the
    /// effective window are filtered out so the plot renders only the
    /// visible slice. Mirrors Swift Charts' `.chartXVisibleDomain` +
    /// `.chartScrollPosition` pair.
    pub(crate) scroll: Option<ChartScrollConfig>,
    /// Optional Audio Graphs descriptor. When set, the `summary` field
    /// fills in the default VoiceOver label (unless a caller-supplied
    /// `accessibility_label` wins), and the descriptor's tone sequence
    /// is reachable via [`ChartView::play_audio_graph`] so hosts can wire
    /// VoiceOver's `VO + Shift + S` shortcut through to sonification.
    ///
    /// [`ChartView::play_audio_graph`]: super::view::ChartView::play_audio_graph
    pub(crate) audio_graph: Option<Arc<ChartDescriptor>>,
}

impl Chart {
    /// Create a new chart for the given series.
    ///
    /// Accepts a [`ChartDataSeries`] (single-series) or [`ChartDataSet`]
    /// (multi-series). The default id is `"chart"`; callers rendering more
    /// than one chart in the same window must override via [`Chart::id`].
    pub fn new(series: impl Into<ChartDataSet>) -> Self {
        Self {
            id: ElementId::Name(SharedString::from("chart")),
            chart_type: ChartType::default(),
            data_set: series.into(),
            width: px(240.0),
            height: px(120.0),
            color: None,
            accessibility_label: None,
            point_focus_group: None,
            point_focus_handles: Vec::new(),
            axis: None,
            gridlines: None,
            show_legend: None,
            legend_position: LegendPosition::default(),
            title: None,
            subtitle: None,
            inner_radius_ratio: 0.0,
            sector_start_angle: DEFAULT_SECTOR_START_ANGLE,
            stacking: MarkStackingMethod::default(),
            bar_orientation: BarOrientation::default(),
            annotations: Vec::new(),
            interpolation: InterpolationMethod::default(),
            scroll: None,
            audio_graph: None,
        }
    }

    /// Override the chart's root element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Set the mark type (Bar, Line, Area, Point, Range, or Rule).
    pub fn chart_type(mut self, chart_type: ChartType) -> Self {
        self.chart_type = chart_type;
        self
    }

    /// Set the chart's overall size. Plot area is this minus any axis
    /// margins and the container's corner-radius inset.
    pub fn size(mut self, width: Pixels, height: Pixels) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Override the single-series mark colour. Multi-series charts auto-
    /// assign palette colours; this value only applies to series index 0.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Override the auto-generated VoiceOver label.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Register a focus group so each data point becomes a Tab stop under
    /// Full Keyboard Access.
    pub fn point_focus_group(mut self, group: FocusGroup) -> Self {
        self.point_focus_group = Some(group);
        self
    }

    /// Supply one [`FocusHandle`] per data point across all series.
    /// The length must equal the sum of series lengths; otherwise focus
    /// registration is skipped.
    pub fn point_focus_handles(mut self, handles: Vec<FocusHandle>) -> Self {
        self.point_focus_handles = handles;
        self
    }

    /// Configure axis labels and tick marks. Without this, the chart
    /// renders as a sparkline with no margins.
    pub fn axis(mut self, config: AxisConfig) -> Self {
        self.axis = Some(config);
        self
    }

    /// Add gridlines to the chart. Rendered behind data marks.
    pub fn gridlines(mut self, config: GridlineConfig) -> Self {
        self.gridlines = Some(config);
        self
    }

    /// Force the legend row on (`true`) or off (`false`).
    ///
    /// Without this, multi-series charts auto-show a legend and
    /// single-series charts hide it. Call `show_legend(true)` to force
    /// a legend on a single-series chart, or `show_legend(false)` to
    /// hide the legend on a multi-series chart.
    pub fn show_legend(mut self, show: bool) -> Self {
        self.show_legend = Some(show);
        self
    }

    /// Position the legend relative to the plot area.
    ///
    /// Mirrors Swift Charts' `.chartLegend(position:)` surface.
    /// [`LegendPosition::Automatic`] (default) places the legend below the
    /// plot for multi-series charts and hides it for single-series charts
    /// unless [`Chart::show_legend`] forces it on. [`LegendPosition::Top`]
    /// and [`LegendPosition::Bottom`] wrap the plot in a vertical column;
    /// [`LegendPosition::Leading`] and [`LegendPosition::Trailing`] wrap it
    /// in a horizontal row. [`LegendPosition::Hidden`] suppresses the
    /// legend even for multi-series charts.
    pub fn legend_position(mut self, position: LegendPosition) -> Self {
        self.legend_position = position;
        self
    }

    /// Add a title above the chart. Per HIG: "Aid comprehension by adding
    /// descriptive text."
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add a subtitle below the title.
    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Ratio of inner radius to outer radius for [`ChartType::Sector`].
    ///
    /// `0.0` (default) renders a solid pie; `0.6` renders a donut with a
    /// 60% hole. Values above `0.95` are clamped so the ring remains
    /// visible.
    pub fn inner_radius_ratio(mut self, ratio: f32) -> Self {
        self.inner_radius_ratio = ratio.clamp(0.0, 0.95);
        self
    }

    /// Start angle (radians) for [`ChartType::Sector`].
    ///
    /// Defaults to −π/2 so the first slice begins at 12 o'clock and
    /// sweeps clockwise.
    pub fn sector_start_angle(mut self, angle: f32) -> Self {
        self.sector_start_angle = angle;
        self
    }

    /// Stack multi-series marks vertically. Applies to Bar and Area charts.
    ///
    /// - [`MarkStackingMethod::Unstacked`] (default): bars sit side-by-side;
    ///   areas overlay each other.
    /// - [`MarkStackingMethod::Standard`]: bars and areas accumulate
    ///   positively from the baseline.
    /// - [`MarkStackingMethod::Normalized`]: each slot's total is forced
    ///   to the plot height (ratios, not magnitudes).
    /// - [`MarkStackingMethod::Center`]: stream-graph centring; each slot's
    ///   stack pivots on its midpoint.
    pub fn stacking(mut self, method: MarkStackingMethod) -> Self {
        self.stacking = method;
        self
    }

    /// Set bar orientation for [`ChartType::Bar`].
    ///
    /// - [`BarOrientation::Vertical`] (default): slots distribute left-to-right
    ///   and bars grow bottom-to-top from the X baseline.
    /// - [`BarOrientation::Horizontal`]: slots distribute top-to-bottom and
    ///   bars grow left-to-right from the Y baseline, matching Swift Charts'
    ///   `BarMark(x: .value(.quantity), y: .value(.category))` orientation.
    ///
    /// Ignored for non-Bar charts.
    pub fn bar_orientation(mut self, orientation: BarOrientation) -> Self {
        self.bar_orientation = orientation;
        self
    }

    /// Pin a set of [`ChartAnnotation`]s to the plot.
    ///
    /// Each annotation targets a data point (by series/point index) or a
    /// raw `(x, y)` value, and renders as an absolute-positioned Text or
    /// Icon above/below/beside/over the mark. Overflow resolution flips
    /// the position toward the plot interior so callouts never sit on
    /// top of the container's rounded clip edge.
    ///
    /// HIG: *Aid comprehension by adding descriptive text* — annotations
    /// mirror Swift Charts' `.annotation()` surface.
    pub fn annotations(mut self, annotations: Vec<ChartAnnotation>) -> Self {
        self.annotations = annotations;
        self
    }

    /// Select the interpolation method used to connect data points for
    /// Line, Area, Range, and stacked-area marks.
    ///
    /// Defaults to [`InterpolationMethod::CatmullRom`] (smooth curve
    /// matching previous releases). Use [`InterpolationMethod::Linear`]
    /// when straight segments are required, [`InterpolationMethod::Monotone`]
    /// for non-negative quantities where cosmetic overshoot would be
    /// misleading, or one of the step variants for discrete-state time
    /// series. Ignored by Bar, Point, Sector, and Rectangle marks.
    ///
    /// Mirrors Swift Charts' `.interpolationMethod(_:)` modifier.
    pub fn interpolation(mut self, method: InterpolationMethod) -> Self {
        self.interpolation = method;
        self
    }

    /// Configure a scroll / zoom window over the X axis.
    ///
    /// Mirrors Swift Charts' `.chartXVisibleDomain(length:)` +
    /// `.chartScrollPosition(initialX:)` pair. When the visible domain is
    /// narrower than the full data extent, points whose numeric X falls
    /// outside the effective window are filtered out and the remainder is
    /// rendered across the full plot width — the on-screen slot count
    /// shrinks, so the visible data reads at a zoomed-in density.
    ///
    /// Only numeric (`PlottableValue::Number`) domains are honoured today.
    /// Date and Category scroll support is a follow-up; the API already
    /// takes the full [`PlottableValue`] so it doesn't need to widen later.
    pub fn scroll(mut self, config: ChartScrollConfig) -> Self {
        self.scroll = Some(config);
        self
    }

    /// Attach an [`ChartDescriptor`] for VoiceOver + Audio Graphs.
    ///
    /// Mirrors Apple's `.accessibilityChartDescriptor(_:)` modifier. The
    /// descriptor's `summary` fills in the default VoiceOver label when no
    /// explicit [`Chart::accessibility_label`] is set — a caller-supplied
    /// label still wins so the explicit override path stays intact. The
    /// descriptor is also reachable via
    /// [`ChartView::play_audio_graph`](super::view::ChartView::play_audio_graph)
    /// so hosts can trigger sonification from their own keybinding or
    /// button.
    ///
    /// HIG: *Consider Audio Graphs for VoiceOver users — an audio
    /// representation of your chart that people can listen to as well as
    /// read.*
    pub fn audio_graph(mut self, descriptor: ChartDescriptor) -> Self {
        self.audio_graph = Some(Arc::new(descriptor));
        self
    }

    /// Attach a shared [`ChartDescriptor`] without re-wrapping.
    ///
    /// Used by [`ChartView`](super::view::ChartView) to thread its
    /// descriptor through to the inner `Chart` each render without
    /// deep-cloning the series / axis data.
    pub(crate) fn audio_graph_arc(mut self, descriptor: Arc<ChartDescriptor>) -> Self {
        self.audio_graph = Some(descriptor);
        self
    }

    /// Build the default VoiceOver label per HIG guidance.
    pub(crate) fn default_accessibility_label(&self) -> String {
        let series_count = self.data_set.series.len();
        if series_count == 0 {
            return format!("{} chart: no series", self.chart_type.voice_label());
        }
        if series_count == 1 {
            let s = &self.data_set.series[0].inner;
            let count = s.points.len();
            if count == 0 {
                return format!(
                    "{} chart: {}, no values",
                    self.chart_type.voice_label(),
                    s.name
                );
            }
            return format!(
                "{} chart: {}, {} values, range {:.2} to {:.2}",
                self.chart_type.voice_label(),
                s.name,
                count,
                s.min_value(),
                s.max_value()
            );
        }
        let names: Vec<&str> = self
            .data_set
            .series
            .iter()
            .map(|s| s.inner.name.as_ref())
            .collect();
        format!(
            "{} chart: {} series ({})",
            self.chart_type.voice_label(),
            series_count,
            names.join(", ")
        )
    }
}

fn build_fka_labels(
    chart_type: ChartType,
    data_set: &ChartDataSet,
    multi_series: bool,
    total: usize,
) -> Arc<[SharedString]> {
    let voice = chart_type.voice_label();
    let mut labels: Vec<SharedString> = Vec::with_capacity(total);
    for series in data_set.series.iter() {
        for p in series.inner.points.iter() {
            // Per-point label is content-only — position is carried
            // structurally by `posinset` / `setsize` on the DataPoint role,
            // so VoiceOver can synthesise "row N of M" on its own. Leaving
            // "N of M" in the label made VoiceOver read it twice.
            let v = p.y.as_number_f32().unwrap_or(0.0);
            let label = if multi_series {
                format!("{}: {v:.2}", series.inner.name)
            } else {
                format!("{voice}: {v:.2}")
            };
            labels.push(SharedString::from(label));
        }
    }
    labels.into()
}

pub(crate) fn palette_color(idx: usize, theme: &crate::foundations::theme::TahoeTheme) -> Hsla {
    let p = &theme.palette;
    match PALETTE[idx % PALETTE.len()] {
        "blue" => p.blue,
        "green" => p.green,
        "orange" => p.orange,
        "purple" => p.purple,
        "pink" => p.pink,
        "teal" => p.teal,
        "red" => p.red,
        "yellow" => p.yellow,
        "cyan" => p.cyan,
        "indigo" => p.indigo,
        "mint" => p.mint,
        "brown" => p.brown,
        _ => theme.accent,
    }
}

/// Resolve the colour for a series at the given index.
///
/// Priority: per-series color > global color (idx 0 only) > palette color.
pub(crate) fn series_color(
    data_set: &ChartDataSet,
    global_color: Option<Hsla>,
    idx: usize,
    theme: &crate::foundations::theme::TahoeTheme,
) -> Hsla {
    if let Some(c) = data_set.series[idx].color {
        return c;
    }
    if idx == 0
        && let Some(c) = global_color
    {
        return c;
    }
    palette_color(idx, theme)
}

impl RenderOnce for Chart {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let dwc = theme.accessibility_mode.differentiate_without_color();

        let total_width = self.width;
        let total_height = self.height;
        let chart_type = self.chart_type;

        // Scroll window: if the config resolves to a narrower numeric X
        // window than the data extent, drop points outside it before any
        // downstream math runs. Everything after this (min/max, offsets,
        // fka labels, paint) treats the filtered set as the dataset, so
        // the plot renders the visible slice at full width.
        if let Some(scroll_cfg) = self.scroll.as_ref()
            && let Some(extent) = x_numeric_extent(&self.data_set)
            && let Some(window) = scroll_cfg.effective_numeric_window(extent)
        {
            self.data_set = filter_data_set_to_window(self.data_set, window);
        }

        let raw_min = self.data_set.global_min();
        let raw_max = self.data_set.global_max();
        // Anchor Bar/Area/Range at zero; let Line/Point/Rule breathe around
        // their actual data so all-negative series still fill the plot area
        // instead of collapsing against the top edge.
        let (mut min, mut max) = if chart_type.anchors_at_zero() {
            (raw_min.min(0.0), raw_max.max(0.0))
        } else {
            (raw_min, raw_max)
        };
        // Fallback for empty / NaN / zero-width ranges. Without this the
        // divisor in `(v - min) / range` would be zero or non-finite.
        if !(min.is_finite() && max.is_finite()) || (max - min).abs() < f32::EPSILON {
            min = min.min(0.0);
            max = (min + 1.0).max(max);
        }

        // VoiceOver label priority: explicit `accessibility_label`, then
        // the Audio Graphs descriptor summary (HIG's "describe the chart"
        // field), then the auto-generated fallback. Callers who set both
        // still get their explicit label — audio_graph's summary is a
        // richer default, not an override.
        let a11y_label: SharedString = match self.accessibility_label.take() {
            Some(label) => label,
            None => match self.audio_graph.as_ref() {
                Some(desc) => desc.summary.clone(),
                None => SharedString::from(self.default_accessibility_label()),
            },
        };
        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::Group);

        let root_id = self.id;

        let total_fka_points: usize = self
            .data_set
            .series
            .iter()
            .map(|s| s.inner.points.len())
            .sum();

        // Prefix-sum offsets so each series occupies a contiguous slice of
        // focus handles regardless of per-series length. Indexing with
        // `si * max_points + slot_i` was wrong for unequal-length series.
        let series_offsets: Vec<usize> = self
            .data_set
            .series
            .iter()
            .scan(0usize, |acc, s| {
                let start = *acc;
                *acc += s.inner.points.len();
                Some(start)
            })
            .collect();

        let fka_points = FocusGroup::bind_if_fka(
            theme.full_keyboard_access(),
            self.point_focus_group,
            self.point_focus_handles,
            total_fka_points,
        );

        let point_prefix: Option<SharedString> = fka_points
            .as_ref()
            .map(|_| SharedString::from(format!("{root_id}-point")));

        let all_empty = self
            .data_set
            .series
            .iter()
            .all(|s| s.inner.points.is_empty());

        let data_set = self.data_set;
        let global_color = self.color;
        let axis_config = self.axis.take();
        let gridline_config = self.gridlines.take();
        // `LegendPosition::Hidden` dominates everything; otherwise
        // `show_legend` (explicit boolean) takes priority, then the
        // multi-series auto-show default.
        let legend_position = self.legend_position;
        let show_legend = if matches!(legend_position, LegendPosition::Hidden) {
            false
        } else {
            self.show_legend.unwrap_or(data_set.is_multi())
        };
        let title = self.title.take();
        let subtitle = self.subtitle.take();
        let multi_series = data_set.is_multi();
        let annotations = std::mem::take(&mut self.annotations);
        let interpolation = self.interpolation;

        // Build per-point VoiceOver labels for this render pass. `Chart`
        // is `RenderOnce` so we can't cache across renders; the win here is
        // one pass per redraw instead of per inner paint-loop iteration.
        // `attach_fka` then takes a slice and clones a cheap `SharedString`
        // (Arc bump) per point, not a `String`.
        let fka_labels: Arc<[SharedString]> = if fka_points.is_some() {
            build_fka_labels(chart_type, &data_set, multi_series, total_fka_points)
        } else {
            Arc::from([] as [SharedString; 0])
        };

        // Sector and Rectangle charts own their plot geometry. Suppress
        // the standard linear axis pipeline so we don't paint stray
        // gridlines across a pie or a heatmap's cell grid.
        let custom_geometry = chart_type.uses_custom_plot_geometry();
        let has_gridlines =
            !custom_geometry && gridline_config.as_ref().is_some_and(|g| g.is_active());

        // Compute axis margins. When no axis is configured, plot area =
        // full container (v1 sparkline mode). `AxisTickStyle::Hidden`
        // collapses the label gutter so trailing padding is reclaimed.
        let y_tick_hidden = axis_config
            .as_ref()
            .is_some_and(|a| matches!(a.y_marks.tick_style, AxisTickStyle::Hidden));
        let has_y_axis = !custom_geometry
            && !y_tick_hidden
            && axis_config.as_ref().is_some_and(|a| {
                a.y_tick_count > 0
                    || a.y_ticks.is_some()
                    || matches!(a.y_marks.tick_style, AxisTickStyle::Manual(_))
            });
        let has_x_labels =
            !custom_geometry && axis_config.as_ref().is_some_and(|a| a.x_labels.is_some());
        let y_position = axis_config
            .as_ref()
            .map(|a| a.effective_y_position())
            .unwrap_or(AxisPosition::Leading);
        let x_position = axis_config
            .as_ref()
            .map(|a| a.effective_x_position())
            .unwrap_or(AxisPosition::Bottom);
        let y_formatter = axis_config
            .as_ref()
            .and_then(|a| a.y_marks.value_label_formatter.clone());
        let y_grid_style_override = axis_config.as_ref().map(|a| a.y_marks.grid_line_style);

        // Compute the effective Y scale once. User-supplied scale wins;
        // otherwise default to a LinearScale over the data extent — that
        // matches the legacy `(v - min) / range` projection byte-for-byte
        // so callers who never touch the scale API see no pixel diff.
        let y_scale: Arc<dyn Scale> = axis_config
            .as_ref()
            .and_then(|a| a.y_scale.clone())
            .unwrap_or_else(|| Arc::new(LinearScale::new(min as f64, max as f64)));

        // Compute Y-tick (value, label) pairs once — shared by the gridline
        // canvas and the Y-label column. When the user supplies a scale,
        // its `ticks()` drives the axis; otherwise we fall back to the
        // existing `nice_ticks` path via `compute_y_ticks`. A custom
        // `value_label_formatter` replaces the default numeric label.
        let y_tick_pairs: Vec<(PlottableValue, SharedString)> = if has_y_axis || has_gridlines {
            let mut pairs = match axis_config.as_ref().and_then(|a| a.y_scale.as_ref()) {
                Some(scale) => {
                    let count = axis_config
                        .as_ref()
                        .map(|a| a.y_tick_count.max(1))
                        .unwrap_or(5);
                    let mut pairs = scale.ticks(count);
                    pairs.sort_by(|a, b| match (a.0.as_number(), b.0.as_number()) {
                        (Some(x), Some(y)) => {
                            x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        _ => std::cmp::Ordering::Equal,
                    });
                    pairs
                }
                None => {
                    let mut ticks = axis_config
                        .as_ref()
                        .map(|a| a.compute_y_ticks(min, max))
                        .unwrap_or_default();
                    ticks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    ticks
                        .into_iter()
                        .map(|v| {
                            (
                                PlottableValue::Number(v as f64),
                                SharedString::from(format_y_tick(v)),
                            )
                        })
                        .collect()
                }
            };
            if let Some(ref fmt) = y_formatter {
                for (value, label) in pairs.iter_mut() {
                    *label = fmt(value);
                }
            }
            pairs
        } else {
            Vec::new()
        };

        let y_margin = if has_y_axis {
            px(AxisConfig::y_label_width(theme))
        } else {
            px(0.0)
        };
        let x_margin = if has_x_labels {
            px(AxisConfig::x_label_height(theme))
        } else {
            px(0.0)
        };

        // Inset the plot area by the container's corner radius on every
        // side. `overflow_hidden` + `rounded` clips the four corners, so
        // painting edge-to-edge (e.g. a Line peak landing on `origin.y`)
        // puts the stroke in the clipped region and the mark appears cut
        // off. Equal inset on all sides keeps the plot rectangle inside
        // the unclipped interior regardless of which corner it would
        // otherwise reach.
        let plot_inset = theme.radius_md;
        let inset_f = f32::from(plot_inset);
        let plot_width =
            px((f32::from(total_width) - 2.0 * inset_f - f32::from(y_margin)).max(0.0));
        let plot_height =
            px((f32::from(total_height) - 2.0 * inset_f - f32::from(x_margin)).max(0.0));

        // Compute stack segments once per render when a stacking mode is
        // active; bars and areas share the same per-(series, slot) table.
        let stacking = self.stacking;
        let stacks: Option<Vec<Vec<StackSegment>>> = if stacking.is_active()
            && matches!(chart_type, ChartType::Bar | ChartType::Area)
            && multi_series
        {
            Some(compute_stacks(&data_set, stacking, min, max))
        } else {
            None
        };

        let bar_orientation = self.bar_orientation;

        // Build the plot area.
        let plot: Option<gpui::Div> = if all_empty {
            None
        } else {
            Some(match chart_type {
                ChartType::Bar => match bar_orientation {
                    BarOrientation::Vertical => render_bars(
                        &data_set,
                        global_color,
                        theme,
                        window,
                        plot_width,
                        plot_height,
                        y_scale.as_ref(),
                        &fka_points,
                        &point_prefix,
                        total_fka_points,
                        &series_offsets,
                        &fka_labels,
                        stacks.as_deref(),
                        dwc,
                    ),
                    BarOrientation::Horizontal => render_horizontal_bars(
                        &data_set,
                        global_color,
                        theme,
                        window,
                        plot_width,
                        plot_height,
                        y_scale.as_ref(),
                        &fka_points,
                        &point_prefix,
                        total_fka_points,
                        &series_offsets,
                        &fka_labels,
                        stacks.as_deref(),
                        dwc,
                    ),
                },
                ChartType::Point => render_points(
                    &data_set,
                    global_color,
                    theme,
                    window,
                    plot_width,
                    plot_height,
                    y_scale.as_ref(),
                    axis_config.as_ref().and_then(|a| a.x_scale.clone()),
                    &fka_points,
                    &point_prefix,
                    total_fka_points,
                    &series_offsets,
                    &fka_labels,
                    dwc,
                ),
                ChartType::Line | ChartType::Area | ChartType::Range | ChartType::Rule => {
                    render_canvas(
                        &data_set,
                        global_color,
                        theme,
                        window,
                        plot_width,
                        plot_height,
                        y_scale.clone(),
                        &fka_points,
                        &point_prefix,
                        total_fka_points,
                        &series_offsets,
                        &fka_labels,
                        chart_type,
                        stacks.clone(),
                        dwc,
                        interpolation,
                    )
                }
                ChartType::Sector => render_sector(
                    &data_set,
                    global_color,
                    theme,
                    plot_width,
                    plot_height,
                    self.inner_radius_ratio,
                    self.sector_start_angle,
                    dwc,
                ),
                ChartType::Rectangle => render_rectangle(
                    &data_set,
                    global_color,
                    theme,
                    plot_width,
                    plot_height,
                    axis_config.as_ref(),
                    dwc,
                ),
            })
        };

        // Build optional gridline canvas layer.
        let max_data_points = data_set.max_points();
        let show_y_line = axis_config.as_ref().is_some_and(|a| a.show_y_line);

        let overlay_canvas = if has_gridlines || show_y_line {
            let gl = gridline_config.as_ref();
            let gl_base_style = gl.map(|g| g.style).unwrap_or_default();
            // Y-axis `grid_line_style` wins when it isn't the default
            // (`Solid`); otherwise fall back to whatever the gridline
            // config requested. This lets a caller set `Dashed` on the
            // gridline config globally but `Hidden` on the Y-axis to
            // suppress horizontal gridlines while keeping vertical ones.
            let h_style = match y_grid_style_override {
                Some(GridLineStyle::Solid) | None => gl_base_style,
                Some(other) => other,
            };
            let show_h =
                gl.is_some_and(|g| g.horizontal) && !matches!(h_style, GridLineStyle::Hidden);
            let show_v =
                gl.is_some_and(|g| g.vertical) && !matches!(gl_base_style, GridLineStyle::Hidden);
            let v_count = if show_v { max_data_points } else { 0 };
            let gl_color = gl.and_then(|g| g.color).unwrap_or(theme.separator_color());
            let y_line_color = theme.text_muted;
            let pw = f32::from(plot_width);
            let ph = f32::from(plot_height);
            let h_ticks: Vec<PlottableValue> =
                y_tick_pairs.iter().map(|(v, _)| v.clone()).collect();
            let scale_for_canvas = y_scale.clone();
            let v_style = gl_base_style;

            // Absolute positioning is load-bearing: the plot wrapper at
            // line ~945 is `div().relative()` (block layout, not flex), so
            // an in-flow gridline canvas would stack above the plot and
            // push bars/lines out of the plot_height region — producing
            // clumped, clipped marks. Absolute + top/left 0 overlays the
            // gridlines behind the plot without displacing it.
            Some(
                canvas(
                    |_info, _window, _cx| {},
                    move |bounds, _state, window, _cx| {
                        let origin = bounds.origin;
                        let bw = f32::from(bounds.size.width);
                        let bh = f32::from(bounds.size.height);
                        if show_h && !h_ticks.is_empty() {
                            super::marks::paint_horizontal_gridlines(
                                window,
                                origin,
                                bw,
                                bh,
                                &h_ticks,
                                scale_for_canvas.as_ref(),
                                gl_color,
                                h_style,
                            );
                        }
                        if show_v && v_count > 1 {
                            super::marks::paint_vertical_gridlines(
                                window, origin, bw, bh, v_count, gl_color, v_style,
                            );
                        }
                        if show_y_line {
                            super::marks::paint_y_axis_line(window, origin, bh, y_line_color);
                        }
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .w(px(pw))
                .h(px(ph)),
            )
        } else {
            None
        };

        // Compose the layout: Y-labels | plot area (with X-labels below).
        // `p(plot_inset)` keeps every child (axis labels and plot area) out
        // of the rounded-corner clip region; `plot_width`/`plot_height`
        // above already account for this so the content fits inside the
        // padded area.
        let mut container = div()
            .w(total_width)
            .h(total_height)
            .bg(theme.surface)
            .rounded(theme.radius_md)
            .border_1()
            .border_color(theme.border)
            .p(plot_inset)
            .overflow_hidden()
            .with_accessibility(&a11y_props);

        if let Some(plot_el) = plot {
            // Annotations need an x-scale so `resolve_annotation` can project
            // `DataPoint` / `Value` targets into pixel space. Use the caller's
            // axis override when present; otherwise a LinearScale over
            // `(-0.5, max_points - 0.5)` so point index `i` lands at the
            // centre of its slot — matching where bars/points render.
            let x_scale_for_annot: Option<Arc<dyn Scale>> = if annotations.is_empty() {
                None
            } else {
                Some(
                    axis_config
                        .as_ref()
                        .and_then(|a| a.x_scale.clone())
                        .unwrap_or_else(|| {
                            let n = max_data_points.max(1);
                            Arc::new(LinearScale::new(-0.5, n as f64 - 0.5))
                        }),
                )
            };

            // Wrap the plot with optional gridline canvas + annotation layer.
            let needs_wrapper = overlay_canvas.is_some() || !annotations.is_empty();
            let plot_wrapper = if needs_wrapper {
                let mut wrapper = div().relative().w(plot_width).h(plot_height);
                if let Some(gl_canvas) = overlay_canvas {
                    wrapper = wrapper.child(gl_canvas);
                }
                wrapper = wrapper.child(plot_el);
                if let Some(xs) = x_scale_for_annot.as_ref() {
                    let pw_f = f32::from(plot_width);
                    let ph_f = f32::from(plot_height);
                    for annotation in &annotations {
                        if let Some((ax, ay, pos)) = resolve_annotation(
                            annotation,
                            &data_set,
                            xs.as_ref(),
                            y_scale.as_ref(),
                            pw_f,
                            ph_f,
                            ANNOTATION_FLIP_THRESHOLD_PX,
                        ) {
                            wrapper = wrapper
                                .child(build_annotation_element(annotation, ax, ay, pos, theme));
                        }
                    }
                }
                wrapper.into_any_element()
            } else {
                plot_el.into_any_element()
            };

            if has_y_axis || has_x_labels {
                let mut top_row = div().flex().flex_row().w_full();

                let y_col = if has_y_axis {
                    let (pad_left, pad_right) = match y_position {
                        AxisPosition::Trailing => (theme.spacing_xs, px(0.0)),
                        _ => (px(0.0), theme.spacing_xs),
                    };
                    let (align, col_pad) = match y_position {
                        AxisPosition::Trailing => (TextAlign::Left, pad_left),
                        _ => (TextAlign::Right, pad_right),
                    };
                    let mut col = div()
                        .flex()
                        .flex_col()
                        .justify_between()
                        .w(y_margin)
                        .h(plot_height);
                    col = match y_position {
                        AxisPosition::Trailing => col.pl(col_pad),
                        _ => col.pr(col_pad),
                    };

                    for (_value, label) in y_tick_pairs.iter().rev() {
                        col = col.child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .text_align(align)
                                .child(label.clone()),
                        );
                    }
                    Some(col)
                } else {
                    None
                };

                if matches!(y_position, AxisPosition::Trailing) {
                    top_row = top_row.child(plot_wrapper);
                    if let Some(col) = y_col {
                        top_row = top_row.child(col);
                    }
                } else {
                    if let Some(col) = y_col {
                        top_row = top_row.child(col);
                    }
                    top_row = top_row.child(plot_wrapper);
                }

                if has_x_labels {
                    let x_labels = axis_config.as_ref().and_then(|a| a.x_labels.clone());
                    if let Some(labels) = x_labels {
                        let max_points = max_data_points.max(1);
                        let (ml, mr) = match y_position {
                            AxisPosition::Trailing => (px(0.0), y_margin),
                            _ => (y_margin, px(0.0)),
                        };
                        let mut x_row = div().flex().flex_row().w_full().h(x_margin).ml(ml).mr(mr);

                        for label in labels.iter().take(max_points) {
                            x_row = x_row.child(
                                div()
                                    .flex_1()
                                    .justify_center()
                                    .text_style(TextStyle::Caption1, theme)
                                    .text_color(theme.text_muted)
                                    .child(label.clone()),
                            );
                        }
                        if matches!(x_position, AxisPosition::Top) {
                            container = container.child(x_row).child(top_row);
                        } else {
                            container = container.child(top_row).child(x_row);
                        }
                    } else {
                        container = container.child(top_row);
                    }
                } else {
                    container = container.child(top_row);
                }
            } else {
                container = container.child(plot_wrapper);
            }
        } else {
            container = container.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w_full()
                    .h_full()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .child("No data"),
            );
        }

        // `show_legend` already encodes auto-hide semantics (None = auto-
        // hide on single-series, `Hidden` forces off). Keep the extra `> 1`
        // guard off so a caller that explicitly forces `show_legend(true)`
        // on a single series still renders the swatch.
        if show_legend && !data_set.series.is_empty() {
            // Leading/Trailing stack swatches vertically so the column stays
            // narrow next to the plot; Top/Bottom/Automatic lay them out in
            // a row beneath/above the chart.
            let vertical_legend = matches!(
                legend_position,
                LegendPosition::Leading | LegendPosition::Trailing
            );
            let mut legend_container = if vertical_legend {
                div()
                    .flex()
                    .flex_col()
                    .gap(theme.spacing_xs)
                    .px(theme.spacing_sm)
                    .py(theme.spacing_xs)
            } else {
                div()
                    .flex()
                    .flex_row()
                    .gap(theme.spacing_sm_md)
                    .px(theme.spacing_sm)
                    .pt(theme.spacing_xs)
                    .w(total_width)
            };

            for (si, series) in data_set.series.iter().enumerate() {
                let color = series_color(&data_set, global_color, si, theme);
                legend_container = legend_container.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(theme.spacing_xs)
                        .child(
                            div()
                                .size(theme.spacing_sm)
                                .rounded(theme.radius_full)
                                .bg(color),
                        )
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .child(series.inner.name.clone()),
                        ),
                );
            }

            let chart_el = match legend_position {
                LegendPosition::Top => div()
                    .flex()
                    .flex_col()
                    .child(legend_container)
                    .child(container),
                LegendPosition::Leading => div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(legend_container)
                    .child(container),
                LegendPosition::Trailing => div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(container)
                    .child(legend_container),
                // Automatic / Bottom / Hidden (Hidden is filtered above).
                _ => div()
                    .flex()
                    .flex_col()
                    .child(container)
                    .child(legend_container),
            };
            wrap_with_title(chart_el, title, subtitle, theme)
        } else {
            wrap_with_title(container, title, subtitle, theme)
        }
    }
}

/// Wrap a chart element with optional title and subtitle above it.
fn wrap_with_title(
    chart_el: gpui::Div,
    title: Option<SharedString>,
    subtitle: Option<SharedString>,
    theme: &crate::foundations::theme::TahoeTheme,
) -> gpui::AnyElement {
    if title.is_none() && subtitle.is_none() {
        return chart_el.into_any_element();
    }
    let mut wrapper = div().flex().flex_col().gap(theme.spacing_xs);
    if let Some(t) = title {
        wrapper = wrapper.child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child(t),
        );
    }
    if let Some(s) = subtitle {
        wrapper = wrapper.child(
            div()
                .text_style(TextStyle::Subheadline, theme)
                .text_color(theme.text_muted)
                .child(s),
        );
    }
    wrapper.child(chart_el).into_any_element()
}

/// Format a Y-axis tick value for display.
fn format_y_tick(v: f32) -> String {
    if v.abs() < 1e-6 {
        return "0".to_string();
    }
    if v.fract() == 0.0 && v.abs() < 1_000_000.0 {
        return format!("{}", v as i64);
    }
    format!("{v:.1}")
}

// ─── Render helpers ─────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn render_bars(
    data_set: &ChartDataSet,
    global_color: Option<Hsla>,
    theme: &crate::foundations::theme::TahoeTheme,
    window: &mut Window,
    width: Pixels,
    height: Pixels,
    y_scale: &dyn Scale,
    fka_points: &Option<(FocusGroup, Vec<FocusHandle>)>,
    point_prefix: &Option<SharedString>,
    total_fka_points: usize,
    series_offsets: &[usize],
    fka_labels: &[SharedString],
    stacks: Option<&[Vec<StackSegment>]>,
    dwc: bool,
) -> gpui::Div {
    if let Some(segs) = stacks {
        return render_stacked_bars(
            data_set,
            global_color,
            theme,
            window,
            width,
            height,
            fka_points,
            point_prefix,
            total_fka_points,
            series_offsets,
            fka_labels,
            segs,
            dwc,
        );
    }

    let n_series = data_set.series.len();
    let max_points = data_set.max_points().max(1);
    let slot_width = f32::from(width) / max_points as f32;

    let bar_count_per_slot = n_series.max(1);
    let total_bar_gap = BAR_GAP * (bar_count_per_slot - 1) as f32;
    let bar_w = bar_width(slot_width, n_series);
    let group_width = bar_w * bar_count_per_slot as f32 + total_bar_gap;
    let group_pad = (slot_width - group_width) / 2.0;

    let mut row = div()
        .flex()
        .flex_row()
        .items_end()
        .w(width)
        .h(height)
        .px(px(group_pad));

    for (slot_i, _point) in (0..max_points).enumerate() {
        let mut group = div()
            .flex()
            .flex_row()
            .items_end()
            .w(px(slot_width))
            .h(height)
            .gap(px(BAR_GAP));

        for (si, series) in data_set.series.iter().enumerate() {
            // Ragged multi-series: a series that runs out of values in this
            // slot renders an empty spacer, not a phantom 0-valued bar —
            // otherwise a short series would bottom-anchor a visible mark
            // at the zero baseline that VoiceOver would also announce.
            let Some(p) = series.inner.points.get(slot_i) else {
                group = group.child(div().w(px(bar_w)));
                continue;
            };
            if p.y.as_number_f32().is_none() {
                group = group.child(div().w(px(bar_w)));
                continue;
            }
            let norm = y_scale.project(&p.y);
            let bar_h = f32::from(height) * norm;
            let color = series_color(data_set, global_color, si, theme);
            let mut bar = div()
                .w(px(bar_w))
                .h(px(bar_h))
                .bg(color)
                .rounded_t(theme.radius_sm);
            if dwc {
                bar = bar.border_1().border_color(theme.text);
            }

            // Contiguous handle offset per series — works correctly when
            // series have different lengths (unlike `si * max_points + slot_i`).
            let fka_idx = series_offsets[si] + slot_i;
            let bar = match (fka_points.as_ref(), point_prefix.as_ref()) {
                (Some((group, handles)), Some(prefix)) if fka_idx < handles.len() => attach_fka(
                    bar,
                    &FkaAttachContext {
                        group,
                        handles,
                        prefix,
                        total: total_fka_points,
                        theme,
                        labels: fka_labels,
                        slot_width,
                    },
                    fka_idx,
                    window,
                ),
                _ => bar.into_any_element(),
            };
            group = group.child(bar);
        }
        row = row.child(group);
    }
    row
}

/// Stacked-bar path: each slot renders a full-width column composed of
/// per-series segments laid out vertically from the baseline.
#[allow(clippy::too_many_arguments)]
fn render_stacked_bars(
    data_set: &ChartDataSet,
    global_color: Option<Hsla>,
    theme: &crate::foundations::theme::TahoeTheme,
    window: &mut Window,
    width: Pixels,
    height: Pixels,
    fka_points: &Option<(FocusGroup, Vec<FocusHandle>)>,
    point_prefix: &Option<SharedString>,
    total_fka_points: usize,
    series_offsets: &[usize],
    fka_labels: &[SharedString],
    stacks: &[Vec<StackSegment>],
    dwc: bool,
) -> gpui::Div {
    let max_points = data_set.max_points().max(1);
    let slot_width = f32::from(width) / max_points as f32;
    let h_f = f32::from(height);
    // Leave a small inter-column gap so adjacent slots remain visually
    // distinct, mirroring the unstacked layout's per-slot gutter.
    let bar_w = (slot_width - BAR_GAP).max(slot_width * 0.6);
    let bar_x_pad = (slot_width - bar_w) * 0.5;

    // Absolute positioning per segment — flex layout struggles with the
    // per-series variable pixel heights when stacking adds non-trivial
    // floating-point drift, so pixel-anchor each segment directly.
    let mut row = div().relative().w(width).h(height);

    for slot_i in 0..max_points {
        for (si, series) in data_set.series.iter().enumerate() {
            let Some(seg) = stacks.get(si).and_then(|s| s.get(slot_i)) else {
                continue;
            };
            // Empty segments (lo == hi) — ragged series, missing values,
            // or negative values clamped to zero — paint nothing.
            if (seg.hi - seg.lo).abs() < f32::EPSILON {
                continue;
            }
            let top = h_f * (1.0 - seg.hi);
            let segment_h = h_f * (seg.hi - seg.lo);
            let color = series_color(data_set, global_color, si, theme);
            let mut bar = div()
                .absolute()
                .top(px(top.max(0.0)))
                .left(px(slot_width * slot_i as f32 + bar_x_pad))
                .w(px(bar_w))
                .h(px(segment_h.max(0.0)))
                .bg(color);
            // Only round the top of the topmost series so adjacent
            // segments read as a single connected column.
            if si + 1 == data_set.series.len() {
                bar = bar.rounded_t(theme.radius_sm);
            }
            if dwc {
                bar = bar.border_1().border_color(theme.text);
            }

            let fka_idx = series_offsets
                .get(si)
                .copied()
                .unwrap_or(0)
                .saturating_add(slot_i);
            if !series.inner.points.is_empty() && slot_i < series.inner.points.len() {
                let bar = match (fka_points.as_ref(), point_prefix.as_ref()) {
                    (Some((group, handles)), Some(prefix)) if fka_idx < handles.len() => {
                        attach_fka(
                            bar,
                            &FkaAttachContext {
                                group,
                                handles,
                                prefix,
                                total: total_fka_points,
                                theme,
                                labels: fka_labels,
                                slot_width,
                            },
                            fka_idx,
                            window,
                        )
                    }
                    _ => bar.into_any_element(),
                };
                row = row.child(bar);
            } else {
                row = row.child(bar);
            }
        }
    }

    row
}

/// Horizontal bar layout: slots distribute top-to-bottom, bars grow
/// left-to-right from the leading edge. Swift Charts calls this
/// `BarMark(x: .value(quantity), y: .value(category))`.
#[allow(clippy::too_many_arguments)]
fn render_horizontal_bars(
    data_set: &ChartDataSet,
    global_color: Option<Hsla>,
    theme: &crate::foundations::theme::TahoeTheme,
    window: &mut Window,
    width: Pixels,
    height: Pixels,
    y_scale: &dyn Scale,
    fka_points: &Option<(FocusGroup, Vec<FocusHandle>)>,
    point_prefix: &Option<SharedString>,
    total_fka_points: usize,
    series_offsets: &[usize],
    fka_labels: &[SharedString],
    stacks: Option<&[Vec<StackSegment>]>,
    dwc: bool,
) -> gpui::Div {
    if let Some(segs) = stacks {
        return render_stacked_horizontal_bars(
            data_set,
            global_color,
            theme,
            window,
            width,
            height,
            fka_points,
            point_prefix,
            total_fka_points,
            series_offsets,
            fka_labels,
            segs,
            dwc,
        );
    }

    let n_series = data_set.series.len();
    let max_points = data_set.max_points().max(1);
    let w_f = f32::from(width);
    let h_f = f32::from(height);
    let slot_height = h_f / max_points as f32;

    let bar_count_per_slot = n_series.max(1);
    let total_bar_gap = BAR_GAP * (bar_count_per_slot - 1) as f32;
    let bar_thickness = bar_width(slot_height, n_series);
    let group_height = bar_thickness * bar_count_per_slot as f32 + total_bar_gap;
    let group_pad = (slot_height - group_height) / 2.0;

    let mut col = div()
        .flex()
        .flex_col()
        .items_start()
        .w(width)
        .h(height)
        .py(px(group_pad));

    for slot_i in 0..max_points {
        let mut group = div()
            .flex()
            .flex_col()
            .items_start()
            .w(width)
            .h(px(slot_height))
            .gap(px(BAR_GAP));

        for (si, series) in data_set.series.iter().enumerate() {
            let Some(p) = series.inner.points.get(slot_i) else {
                group = group.child(div().h(px(bar_thickness)));
                continue;
            };
            if p.y.as_number_f32().is_none() {
                group = group.child(div().h(px(bar_thickness)));
                continue;
            }
            let norm = y_scale.project(&p.y);
            let bar_w = w_f * norm;
            let color = series_color(data_set, global_color, si, theme);
            let mut bar = div()
                .w(px(bar_w))
                .h(px(bar_thickness))
                .bg(color)
                .rounded_r(theme.radius_sm);
            if dwc {
                bar = bar.border_1().border_color(theme.text);
            }

            let fka_idx = series_offsets[si] + slot_i;
            let bar = match (fka_points.as_ref(), point_prefix.as_ref()) {
                (Some((group, handles)), Some(prefix)) if fka_idx < handles.len() => attach_fka(
                    bar,
                    &FkaAttachContext {
                        group,
                        handles,
                        prefix,
                        total: total_fka_points,
                        theme,
                        labels: fka_labels,
                        slot_width: slot_height,
                    },
                    fka_idx,
                    window,
                ),
                _ => bar.into_any_element(),
            };
            group = group.child(bar);
        }
        col = col.child(group);
    }
    col
}

/// Stacked horizontal bars — transpose of [`render_stacked_bars`].
#[allow(clippy::too_many_arguments)]
fn render_stacked_horizontal_bars(
    data_set: &ChartDataSet,
    global_color: Option<Hsla>,
    theme: &crate::foundations::theme::TahoeTheme,
    window: &mut Window,
    width: Pixels,
    height: Pixels,
    fka_points: &Option<(FocusGroup, Vec<FocusHandle>)>,
    point_prefix: &Option<SharedString>,
    total_fka_points: usize,
    series_offsets: &[usize],
    fka_labels: &[SharedString],
    stacks: &[Vec<StackSegment>],
    dwc: bool,
) -> gpui::Div {
    let max_points = data_set.max_points().max(1);
    let w_f = f32::from(width);
    let h_f = f32::from(height);
    let slot_height = h_f / max_points as f32;
    let bar_thickness = (slot_height - BAR_GAP).max(slot_height * 0.6);
    let bar_y_pad = (slot_height - bar_thickness) * 0.5;

    let mut row = div().relative().w(width).h(height);

    for slot_i in 0..max_points {
        for (si, series) in data_set.series.iter().enumerate() {
            let Some(seg) = stacks.get(si).and_then(|s| s.get(slot_i)) else {
                continue;
            };
            if (seg.hi - seg.lo).abs() < f32::EPSILON {
                continue;
            }
            let left = w_f * seg.lo;
            let segment_w = w_f * (seg.hi - seg.lo);
            let color = series_color(data_set, global_color, si, theme);
            let mut bar = div()
                .absolute()
                .top(px(slot_height * slot_i as f32 + bar_y_pad))
                .left(px(left.max(0.0)))
                .w(px(segment_w.max(0.0)))
                .h(px(bar_thickness))
                .bg(color);
            // Round only the trailing edge of the topmost series so
            // adjacent segments read as a single connected row.
            if si + 1 == data_set.series.len() {
                bar = bar.rounded_r(theme.radius_sm);
            }
            if dwc {
                bar = bar.border_1().border_color(theme.text);
            }

            let fka_idx = series_offsets
                .get(si)
                .copied()
                .unwrap_or(0)
                .saturating_add(slot_i);
            if !series.inner.points.is_empty() && slot_i < series.inner.points.len() {
                let bar = match (fka_points.as_ref(), point_prefix.as_ref()) {
                    (Some((group, handles)), Some(prefix)) if fka_idx < handles.len() => {
                        attach_fka(
                            bar,
                            &FkaAttachContext {
                                group,
                                handles,
                                prefix,
                                total: total_fka_points,
                                theme,
                                labels: fka_labels,
                                slot_width: slot_height,
                            },
                            fka_idx,
                            window,
                        )
                    }
                    _ => bar.into_any_element(),
                };
                row = row.child(bar);
            } else {
                row = row.child(bar);
            }
        }
    }

    row
}

#[allow(clippy::too_many_arguments)]
fn render_points(
    data_set: &ChartDataSet,
    global_color: Option<Hsla>,
    theme: &crate::foundations::theme::TahoeTheme,
    window: &mut Window,
    width: Pixels,
    height: Pixels,
    y_scale: &dyn Scale,
    x_scale: Option<Arc<dyn Scale>>,
    fka_points: &Option<(FocusGroup, Vec<FocusHandle>)>,
    point_prefix: &Option<SharedString>,
    total_fka_points: usize,
    series_offsets: &[usize],
    fka_labels: &[SharedString],
    dwc: bool,
) -> gpui::Div {
    // When an X scale is supplied, points land at their real data-space X
    // coordinate (true 2D scatter). Without one, fall back to the legacy
    // index-centred slot layout so existing sparkline-style Point charts
    // render identically.
    if let Some(xs) = x_scale {
        return render_scatter(
            data_set,
            global_color,
            theme,
            window,
            width,
            height,
            y_scale,
            xs.as_ref(),
            fka_points,
            point_prefix,
            total_fka_points,
            series_offsets,
            fka_labels,
            dwc,
        );
    }

    let max_points = data_set.max_points().max(1);
    let slot_width = f32::from(width) / max_points as f32;
    let point_sz = point_size(slot_width);

    let mut row = div().flex().flex_row().items_end().w(width).h(height);

    for (slot_i, _point) in (0..max_points).enumerate() {
        let mut cell = div().w(px(slot_width)).h(height).relative();

        for (si, series) in data_set.series.iter().enumerate() {
            // Ragged multi-series: skip slots past this series' own length so
            // a short series doesn't plant a phantom dot at zero for every
            // trailing slot.
            let Some(p) = series.inner.points.get(slot_i) else {
                continue;
            };
            if p.y.as_number_f32().is_none() {
                continue;
            }
            let norm = y_scale.project(&p.y);
            let top_offset = f32::from(height) * (1.0 - norm) - point_sz / 2.0;
            let color = series_color(data_set, global_color, si, theme);

            let dot = div()
                .absolute()
                .top(px(top_offset.max(0.0)))
                .left(px((slot_width - point_sz) / 2.0))
                .size(px(point_sz))
                .bg(color);

            // Route the per-series shape/fill through apply_marker_shape so
            // outlined markers inherit the series colour for their ring.
            // The DwC border (above the fill) is applied after so it stacks
            // on top of the filled shape.
            let mut dot = apply_marker_shape(dot, si, point_sz, theme, color);

            if dwc {
                dot = dot.border_1().border_color(theme.text);
            }

            let fka_idx = series_offsets[si] + slot_i;
            let dot = match (fka_points.as_ref(), point_prefix.as_ref()) {
                (Some((group, handles)), Some(prefix)) if fka_idx < handles.len() => attach_fka(
                    dot,
                    &FkaAttachContext {
                        group,
                        handles,
                        prefix,
                        total: total_fka_points,
                        theme,
                        labels: fka_labels,
                        slot_width,
                    },
                    fka_idx,
                    window,
                ),
                _ => dot.into_any_element(),
            };
            cell = cell.child(dot);
        }
        row = row.child(cell);
    }
    row
}

/// True 2D scatter — points land at `(x_scale.project(p.x), y_scale.project(p.y))`.
///
/// Slot-free: every point is absolute-positioned against the plot area.
/// A slot_width surrogate (width / n_points) is still computed for FKA
/// focus-ring sizing so keyboard navigation highlights a sensible hit area
/// around each dot.
#[allow(clippy::too_many_arguments)]
fn render_scatter(
    data_set: &ChartDataSet,
    global_color: Option<Hsla>,
    theme: &crate::foundations::theme::TahoeTheme,
    window: &mut Window,
    width: Pixels,
    height: Pixels,
    y_scale: &dyn Scale,
    x_scale: &dyn Scale,
    fka_points: &Option<(FocusGroup, Vec<FocusHandle>)>,
    point_prefix: &Option<SharedString>,
    total_fka_points: usize,
    series_offsets: &[usize],
    fka_labels: &[SharedString],
    dwc: bool,
) -> gpui::Div {
    let w_f = f32::from(width);
    let h_f = f32::from(height);
    let n_points_for_size = data_set
        .series
        .iter()
        .map(|s| s.inner.points.len())
        .max()
        .unwrap_or(1)
        .max(1);
    // Scatter has no slots, so approximate a slot_width for `point_size`
    // from the total point count. This keeps dot sizing proportional to
    // density like the v1 sparkline path.
    let slot_width_surrogate = w_f / n_points_for_size as f32;
    let point_sz = point_size(slot_width_surrogate);

    let mut plot = div().relative().w(width).h(height);

    for (si, series) in data_set.series.iter().enumerate() {
        let color = series_color(data_set, global_color, si, theme);
        for (slot_i, p) in series.inner.points.iter().enumerate() {
            if p.y.as_number_f32().is_none() {
                continue;
            }
            let nx = x_scale.project(&p.x);
            let ny = y_scale.project(&p.y);
            let cx = w_f * nx;
            let cy = h_f * (1.0 - ny);
            let dot = div()
                .absolute()
                .top(px((cy - point_sz * 0.5).max(0.0)))
                .left(px((cx - point_sz * 0.5).max(0.0)))
                .size(px(point_sz))
                .bg(color);
            let mut dot = apply_marker_shape(dot, si, point_sz, theme, color);
            if dwc {
                dot = dot.border_1().border_color(theme.text);
            }

            let fka_idx = series_offsets[si] + slot_i;
            let dot = match (fka_points.as_ref(), point_prefix.as_ref()) {
                (Some((group, handles)), Some(prefix)) if fka_idx < handles.len() => attach_fka(
                    dot,
                    &FkaAttachContext {
                        group,
                        handles,
                        prefix,
                        total: total_fka_points,
                        theme,
                        labels: fka_labels,
                        slot_width: slot_width_surrogate,
                    },
                    fka_idx,
                    window,
                ),
                _ => dot.into_any_element(),
            };
            plot = plot.child(dot);
        }
    }
    plot
}

/// Apply a per-series marker treatment so charts remain distinguishable
/// without colour. Six encodings are interleaved across series:
///
/// | `si % 6` | shape               | treatment      |
/// |----------|---------------------|----------------|
/// | 0        | circle              | solid          |
/// | 1        | square              | solid          |
/// | 2        | rounded square      | solid          |
/// | 3        | circle ring         | outlined       |
/// | 4        | square ring         | outlined       |
/// | 5        | rounded square ring | outlined       |
///
/// Three geometric shapes plus an orthogonal solid/outlined axis covers
/// six distinct encodings without introducing rotated or clipped shapes
/// that GPUI `div` cannot express without a canvas draw.
///
/// `color` is the series' fill colour. Outlined markers draw their ring
/// in the series colour (so the legend swatch still matches the mark)
/// rather than the generic `theme.text` — the previous implementation
/// collapsed every outlined series onto the same foreground colour,
/// defeating the point of the shape/outline rotation.
fn apply_marker_shape(
    dot: gpui::Div,
    si: usize,
    point_size: f32,
    theme: &crate::foundations::theme::TahoeTheme,
    color: Hsla,
) -> gpui::Div {
    let shape_slot = si % 3;
    let outlined = (si / 3) % 2 == 1;
    let shaped = match shape_slot {
        0 => dot.rounded(theme.radius_full),
        1 => dot.rounded(px(1.0)),
        _ => dot.rounded(theme.radius_sm),
    };
    if outlined {
        // Knock out the fill to leave a ring; `bg(transparent())` would
        // drop the colour entirely, so render the ring by inverting the
        // fill to the surface colour and stroking the edge in the series
        // colour via an inset shadow-style border.
        shaped
            .bg(theme.surface)
            .border(px((point_size * 0.22).max(1.0)))
            .border_color(color)
    } else {
        shaped
    }
}

#[allow(clippy::too_many_arguments)]
fn render_canvas(
    data_set: &ChartDataSet,
    global_color: Option<Hsla>,
    theme: &crate::foundations::theme::TahoeTheme,
    window: &mut Window,
    width: Pixels,
    height: Pixels,
    y_scale: Arc<dyn Scale>,
    fka_points: &Option<(FocusGroup, Vec<FocusHandle>)>,
    point_prefix: &Option<SharedString>,
    total_fka_points: usize,
    series_offsets: &[usize],
    fka_labels: &[SharedString],
    chart_type: ChartType,
    stacks: Option<Vec<Vec<StackSegment>>>,
    dwc: bool,
    interpolation: InterpolationMethod,
) -> gpui::Div {
    let w_f = f32::from(width);
    let h_f = f32::from(height);

    let series_data: Vec<(Arc<[super::types::ChartPoint]>, Hsla)> = data_set
        .series
        .iter()
        .enumerate()
        .map(|(si, s)| {
            (
                s.inner.points.clone(),
                series_color(data_set, global_color, si, theme),
            )
        })
        .collect();

    let max_points = data_set.max_points();
    let scale_for_paint = y_scale.clone();
    let scale_for_overlay = y_scale;
    let use_stacked_area = stacks.is_some() && matches!(chart_type, ChartType::Area);
    let stacked_segments = stacks;

    let canvas_el = canvas(
        |_info, _window, _cx| {},
        move |bounds, _state, window, _cx| {
            let origin = bounds.origin;
            let bw = f32::from(bounds.size.width);
            let bh = f32::from(bounds.size.height);
            if use_stacked_area {
                let segs = stacked_segments
                    .as_deref()
                    .expect("use_stacked_area implies stacks are Some");
                for ((points, color), series_segs) in series_data.iter().zip(segs.iter()) {
                    paint_stacked_area(
                        window,
                        origin,
                        bw,
                        bh,
                        points,
                        series_segs,
                        *color,
                        interpolation,
                    );
                }
                return;
            }
            for (points, color) in series_data {
                if let Some(paint) = canvas_paint_callback(
                    chart_type,
                    origin,
                    bw,
                    bh,
                    points,
                    scale_for_paint.clone(),
                    color,
                    interpolation,
                ) {
                    paint(window);
                }
            }
        },
    )
    .w(width)
    .h(height);

    if let (Some((group, handles)), Some(prefix)) = (fka_points.as_ref(), point_prefix.as_ref()) {
        let slot_width = w_f / max_points.max(1) as f32;
        let point_sz = point_size(slot_width);

        // Absolute-position one hit div per (series, slot). A flex_row laid
        // the same slots for every series end-to-end, so a two-series chart
        // compressed each slot to half its width and the second series' hit
        // region sat off the right edge of the plot area. Absolute
        // positioning anchors every (si, slot_i) hit to its true x coord.
        let mut overlay = div().absolute().top_0().left_0().w(width).h(height);

        for (si, series) in data_set.series.iter().enumerate() {
            for (slot_i, p) in series.inner.points.iter().enumerate() {
                // Contiguous handle offset — indexing as `si * max_points +
                // slot_i` is wrong for unequal-length series.
                let fka_idx = series_offsets[si] + slot_i;
                if fka_idx >= handles.len() {
                    break;
                }
                let mut hit = div()
                    .absolute()
                    .top_0()
                    .left(px(slot_width * slot_i as f32))
                    .w(px(slot_width))
                    .h(height);
                if dwc {
                    let norm = scale_for_overlay.project(&p.y);
                    let top_offset = h_f * (1.0 - norm) - point_sz / 2.0;
                    // Transparent ring overlay on top of the canvas mark;
                    // the shape rotates per series so ≥4-series charts stay
                    // distinguishable without colour.
                    let indicator = div()
                        .absolute()
                        .top(px(top_offset.max(0.0)))
                        .left(px((slot_width - point_sz) / 2.0))
                        .size(px(point_sz))
                        .border_1()
                        .border_color(theme.text);
                    let indicator = match si % 3 {
                        0 => indicator.rounded(theme.radius_full),
                        1 => indicator.rounded(px(1.0)),
                        _ => indicator.rounded(theme.radius_sm),
                    };
                    hit = hit.child(indicator);
                }
                let hit = attach_fka(
                    hit,
                    &FkaAttachContext {
                        group,
                        handles,
                        prefix,
                        total: total_fka_points,
                        theme,
                        labels: fka_labels,
                        slot_width,
                    },
                    fka_idx,
                    window,
                );
                overlay = overlay.child(hit);
            }
        }

        div()
            .relative()
            .w(width)
            .h(height)
            .child(canvas_el)
            .child(overlay)
    } else {
        div().w(width).h(height).child(canvas_el)
    }
}

#[allow(clippy::too_many_arguments)]
fn render_sector(
    data_set: &ChartDataSet,
    global_color: Option<Hsla>,
    theme: &crate::foundations::theme::TahoeTheme,
    width: Pixels,
    height: Pixels,
    inner_radius_ratio: f32,
    start_angle: f32,
    dwc: bool,
) -> gpui::Div {
    let weights = sector_weights(data_set, |i| series_color(data_set, global_color, i, theme));
    let pw = f32::from(width);
    let ph = f32::from(height);

    div().w(width).h(height).child(
        canvas(
            |_info, _window, _cx| {},
            move |bounds, _state, window, _cx| {
                paint_sector_chart(
                    window,
                    bounds.origin,
                    pw,
                    ph,
                    &weights,
                    inner_radius_ratio,
                    start_angle,
                    dwc,
                );
            },
        )
        .w(width)
        .h(height),
    )
}

#[allow(clippy::too_many_arguments)]
fn render_rectangle(
    data_set: &ChartDataSet,
    global_color: Option<Hsla>,
    theme: &crate::foundations::theme::TahoeTheme,
    width: Pixels,
    height: Pixels,
    axis_config: Option<&AxisConfig>,
    dwc: bool,
) -> gpui::Div {
    let base_color = series_color(data_set, global_color, 0, theme);
    let pw = f32::from(width);
    let ph = f32::from(height);

    // Rectangle charts need both X and Y scales. Fall back to the data
    // extent when the caller didn't supply one — using `LinearScale` on
    // the raw numeric bounds produces a sensible default grid.
    let x_scale: Arc<dyn Scale> =
        axis_config
            .and_then(|a| a.x_scale.clone())
            .unwrap_or_else(|| {
                let (lo, hi) = rect_x_extent(data_set);
                Arc::new(LinearScale::new(lo as f64, hi as f64))
            });
    let y_scale: Arc<dyn Scale> =
        axis_config
            .and_then(|a| a.y_scale.clone())
            .unwrap_or_else(|| {
                let (lo, hi) = rect_y_extent(data_set);
                Arc::new(LinearScale::new(lo as f64, hi as f64))
            });

    // Slot counts come from the scale tick counts so CategoryScale /
    // DateScale drive cell sizing naturally.
    let x_slots = x_scale.ticks(12).len().max(1);
    let y_slots = y_scale.ticks(12).len().max(1);

    let (cells, z_min, z_max) = build_cells(
        data_set,
        x_scale.as_ref(),
        y_scale.as_ref(),
        pw,
        ph,
        x_slots,
        y_slots,
    );

    div().w(width).h(height).child(
        canvas(
            |_info, _window, _cx| {},
            move |bounds, _state, window, _cx| {
                paint_rectangle_chart(
                    window,
                    bounds.origin,
                    pw,
                    ph,
                    &cells,
                    z_min,
                    z_max,
                    base_color,
                    dwc,
                );
            },
        )
        .w(width)
        .h(height),
    )
}

/// X extent for rectangle charts — reads the first numeric `x` from each
/// point and folds to a `(min, max)` pair. Falls back to `(0, 1)` when no
/// numeric values are present.
fn rect_x_extent(data_set: &ChartDataSet) -> (f32, f32) {
    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;
    for series in data_set.series.iter() {
        for p in series.inner.points.iter() {
            if let Some(x) = p.x.as_number_f32() {
                lo = lo.min(x);
                hi = hi.max(x);
            }
        }
    }
    if lo.is_finite() && hi.is_finite() && (hi - lo).abs() > f32::EPSILON {
        (lo, hi)
    } else {
        (0.0, 1.0)
    }
}

/// Build the absolute-positioned overlay element for a single annotation.
///
/// `(anchor_x, anchor_y)` is the plot-pixel coordinate of the mark the
/// annotation is pinned to; `position` is the overflow-adjusted side
/// returned by [`resolve_annotation`]. The annotation is sized from a
/// coarse character-width estimate (Caption1 ~6 px per char, 16 px tall)
/// because GPUI does not expose a measure-before-paint path — the
/// positions are approximate but stay anchored to the correct side of
/// the mark even when the text rendering differs slightly from the
/// estimate.
fn build_annotation_element(
    annotation: &ChartAnnotation,
    anchor_x: f32,
    anchor_y: f32,
    position: AnnotationPosition,
    theme: &crate::foundations::theme::TahoeTheme,
) -> gpui::AnyElement {
    let (est_w, est_h) = match &annotation.content {
        AnnotationContent::Text(s) => (
            (s.chars().count() as f32 * ANNOTATION_CHAR_W_PX).max(ANNOTATION_MIN_W_PX),
            ANNOTATION_LINE_H_PX,
        ),
        AnnotationContent::Icon(_) => {
            let s = f32::from(theme.icon_size_inline);
            (s, s)
        }
    };

    let (left_px, top_px) = match position {
        AnnotationPosition::Top => (anchor_x - est_w * 0.5, anchor_y - est_h - ANNOTATION_GAP_PX),
        AnnotationPosition::Bottom => (anchor_x - est_w * 0.5, anchor_y + ANNOTATION_GAP_PX),
        AnnotationPosition::Leading => {
            (anchor_x - est_w - ANNOTATION_GAP_PX, anchor_y - est_h * 0.5)
        }
        AnnotationPosition::Trailing => (anchor_x + ANNOTATION_GAP_PX, anchor_y - est_h * 0.5),
        AnnotationPosition::Overlay => (anchor_x - est_w * 0.5, anchor_y - est_h * 0.5),
    };

    let a11y_label: SharedString = match &annotation.content {
        AnnotationContent::Text(s) => s.clone(),
        AnnotationContent::Icon(icon) => SharedString::from(format!("{icon:?}")),
    };
    let props = AccessibilityProps::new()
        .label(a11y_label)
        .role(AccessibilityRole::Group);

    let shell = div()
        .absolute()
        .left(px(left_px.max(0.0)))
        .top(px(top_px.max(0.0)))
        .with_accessibility(&props);

    match &annotation.content {
        AnnotationContent::Text(s) => shell
            .text_style(TextStyle::Caption1, theme)
            .text_color(theme.text_muted)
            .child(s.clone())
            .into_any_element(),
        AnnotationContent::Icon(icon) => shell
            .child(Icon::new(*icon).size(theme.icon_size_inline))
            .into_any_element(),
    }
}

/// Y extent for rectangle charts, same logic as [`rect_x_extent`].
fn rect_y_extent(data_set: &ChartDataSet) -> (f32, f32) {
    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;
    for series in data_set.series.iter() {
        for p in series.inner.points.iter() {
            if let Some(y) = p.y.as_number_f32() {
                lo = lo.min(y);
                hi = hi.max(y);
            }
        }
    }
    if lo.is_finite() && hi.is_finite() && (hi - lo).abs() > f32::EPSILON {
        (lo, hi)
    } else {
        (0.0, 1.0)
    }
}

/// Numeric extent of every series' X values. Skips non-numeric points;
/// returns `None` when the dataset has no numeric X at all (e.g. a pure
/// Category/Date dataset). Used by [`Chart::scroll`] to resolve an
/// effective visible window against the full data extent.
fn x_numeric_extent(data_set: &ChartDataSet) -> Option<(f64, f64)> {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for series in data_set.series.iter() {
        for p in series.inner.points.iter() {
            if let Some(xv) = p.x.as_number() {
                lo = lo.min(xv);
                hi = hi.max(xv);
            }
        }
    }
    if lo.is_finite() && hi.is_finite() {
        Some((lo, hi))
    } else {
        None
    }
}

/// Drop points whose numeric X falls outside the inclusive `(lo, hi)`
/// window. Non-numeric points are dropped too — scroll is currently a
/// numeric-only feature, and points with Date/Category X can't be
/// meaningfully compared to a numeric window.
fn filter_data_set_to_window(data_set: ChartDataSet, window: (f64, f64)) -> ChartDataSet {
    let (lo, hi) = window;
    let series = data_set
        .series
        .into_iter()
        .map(|series| {
            let filtered: Arc<[ChartPoint]> = series
                .inner
                .points
                .iter()
                .filter(|p| p.x.as_number().is_some_and(|xv| xv >= lo && xv <= hi))
                .cloned()
                .collect();
            ChartSeries {
                inner: ChartDataSeries {
                    name: series.inner.name,
                    points: filtered,
                },
                color: series.color,
            }
        })
        .collect();
    ChartDataSet { series }
}
