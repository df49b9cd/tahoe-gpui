//! Chart component render implementation.

use gpui::prelude::*;
use gpui::{
    App, ElementId, FocusHandle, Hsla, Pixels, SharedString, TextAlign, Window, canvas, div, px,
};

use crate::foundations::accessibility::{
    AccessibilityProps, AccessibilityRole, AccessibleExt, FocusGroup,
};
use crate::foundations::theme::ActiveTheme;
use crate::foundations::typography::{TextStyle, TextStyledExt};

use super::accessibility::{FkaAttachContext, attach_fka};
use super::marks::canvas_paint_callback;
#[allow(unused_imports)] // Used in doc links.
use super::types::ChartDataSeries;
use super::types::{
    AxisConfig, BAR_GAP, BAR_WIDTH_RATIO, ChartDataSet, ChartType, GridlineConfig, MAX_POINT_SIZE,
    MIN_POINT_SIZE, TITLE_GAP,
};

/// Palette order for auto-assigned multi-series colours.
const PALETTE: &[&str] = &[
    "blue", "green", "orange", "purple", "pink", "teal", "red", "yellow", "cyan", "indigo", "mint",
    "brown",
];

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
    pub(crate) show_legend: bool,
    pub(crate) title: Option<SharedString>,
    pub(crate) subtitle: Option<SharedString>,
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
            show_legend: false,
            title: None,
            subtitle: None,
        }
    }

    /// Override the chart's root element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn chart_type(mut self, chart_type: ChartType) -> Self {
        self.chart_type = chart_type;
        self
    }

    pub fn size(mut self, width: Pixels, height: Pixels) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn point_focus_group(mut self, group: FocusGroup) -> Self {
        self.point_focus_group = Some(group);
        self
    }

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

    /// Show a legend row below the chart. Automatically shown for
    /// multi-series charts; call this to override.
    pub fn show_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
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

    /// Build the default VoiceOver label per HIG guidance.
    pub(crate) fn default_accessibility_label(&self) -> String {
        let series_count = self.data_set.series.len();
        if series_count == 0 {
            return format!("{} chart: no series", self.chart_type.voice_label());
        }
        if series_count == 1 {
            let s = &self.data_set.series[0].inner;
            let count = s.values.len();
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

    /// Resolve the colour for a series index.
    fn series_color(
        data_set: &ChartDataSet,
        global_color: Option<Hsla>,
        idx: usize,
        theme: &crate::foundations::theme::TahoeTheme,
    ) -> Hsla {
        series_color(data_set, global_color, idx, theme)
    }
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

        let raw_min = self.data_set.global_min();
        let raw_max = self.data_set.global_max();
        let min = if chart_type.anchors_at_zero() {
            raw_min.min(0.0)
        } else {
            raw_min
        };
        let max = raw_max.max(1e-3);
        let range = (max - min).max(1e-3);

        let a11y_label: SharedString = match self.accessibility_label.take() {
            Some(label) => label,
            None => SharedString::from(self.default_accessibility_label()),
        };
        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::Group);

        let root_id = self.id;

        let total_fka_points: usize = self
            .data_set
            .series
            .iter()
            .map(|s| s.inner.values.len())
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
                *acc += s.inner.values.len();
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
            .all(|s| s.inner.values.is_empty());

        let data_set = self.data_set;
        let global_color = self.color;
        let axis_config = self.axis.take();
        let gridline_config = self.gridlines.take();
        let show_legend = self.show_legend || data_set.is_multi();
        let title = self.title.take();
        let subtitle = self.subtitle.take();
        let multi_series = data_set.is_multi();

        let has_gridlines = gridline_config.as_ref().is_some_and(|g| g.is_active());

        // Compute axis margins. When no axis is configured, plot area =
        // full container (v1 sparkline mode).
        let has_y_axis = axis_config
            .as_ref()
            .is_some_and(|a| a.y_tick_count > 0 || a.y_ticks.is_some());
        let has_x_labels = axis_config.as_ref().is_some_and(|a| a.x_labels.is_some());

        // Compute Y-ticks once — shared by the gridline canvas and the
        // Y-label column. Sort defensively so caller-supplied y_ticks(…)
        // render top-to-bottom even when passed out of order.
        let y_ticks: Vec<f32> = if has_y_axis || has_gridlines {
            let mut ticks = axis_config
                .as_ref()
                .map(|a| a.compute_y_ticks(min, max))
                .unwrap_or_default();
            ticks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            ticks
        } else {
            Vec::new()
        };

        let y_margin = if has_y_axis {
            px(AxisConfig::Y_LABEL_WIDTH)
        } else {
            px(0.0)
        };
        let x_margin = if has_x_labels {
            px(AxisConfig::X_LABEL_HEIGHT)
        } else {
            px(0.0)
        };

        let plot_width = px(f32::from(total_width) - f32::from(y_margin));
        let plot_height = px(f32::from(total_height) - f32::from(x_margin));

        // Build the plot area.
        let plot: Option<gpui::Div> = if all_empty {
            None
        } else {
            Some(match chart_type {
                ChartType::Bar => render_bars(
                    &data_set,
                    global_color,
                    theme,
                    window,
                    plot_width,
                    plot_height,
                    min,
                    range,
                    &fka_points,
                    &point_prefix,
                    total_fka_points,
                    &series_offsets,
                    multi_series,
                    chart_type,
                    dwc,
                ),
                ChartType::Point => render_points(
                    &data_set,
                    global_color,
                    theme,
                    window,
                    plot_width,
                    plot_height,
                    min,
                    range,
                    &fka_points,
                    &point_prefix,
                    total_fka_points,
                    &series_offsets,
                    multi_series,
                    chart_type,
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
                        min,
                        range,
                        &fka_points,
                        &point_prefix,
                        total_fka_points,
                        &series_offsets,
                        multi_series,
                        chart_type,
                        dwc,
                    )
                }
            })
        };

        // Build optional gridline canvas layer.
        let max_data_points = data_set
            .series
            .iter()
            .map(|s| s.inner.values.len())
            .max()
            .unwrap_or(0);
        let show_y_line = axis_config.as_ref().is_some_and(|a| a.show_y_line);

        let overlay_canvas = if has_gridlines || show_y_line {
            let gl = gridline_config.as_ref();
            let show_h = gl.is_some_and(|g| g.horizontal);
            let show_v = gl.is_some_and(|g| g.vertical);
            let v_count = if show_v { max_data_points } else { 0 };
            let gl_color = gl.and_then(|g| g.color).unwrap_or(theme.separator_color());
            let y_line_color = theme.text_muted;
            let pw = f32::from(plot_width);
            let ph = f32::from(plot_height);
            let h_ticks = y_ticks.clone();

            Some(
                canvas(
                    |_info, _window, _cx| {},
                    move |bounds, _state, window, _cx| {
                        let origin = bounds.origin;
                        let bw = f32::from(bounds.size.width);
                        let bh = f32::from(bounds.size.height);
                        if show_h && !h_ticks.is_empty() {
                            super::marks::paint_horizontal_gridlines(
                                window, origin, bw, bh, &h_ticks, min, range, gl_color,
                            );
                        }
                        if show_v && v_count > 1 {
                            super::marks::paint_vertical_gridlines(
                                window, origin, bw, bh, v_count, gl_color,
                            );
                        }
                        if show_y_line {
                            super::marks::paint_y_axis_line(window, origin, bh, y_line_color);
                        }
                    },
                )
                .w(px(pw))
                .h(px(ph)),
            )
        } else {
            None
        };

        // Compose the layout: Y-labels | plot area (with X-labels below).
        let mut container = div()
            .w(total_width)
            .h(total_height)
            .bg(theme.surface)
            .rounded(theme.radius_md)
            .border_1()
            .border_color(theme.border)
            .overflow_hidden()
            .with_accessibility(&a11y_props);

        if let Some(plot_el) = plot {
            // Wrap the plot with optional gridline canvas.
            let plot_wrapper = if let Some(gl_canvas) = overlay_canvas {
                div()
                    .relative()
                    .w(plot_width)
                    .h(plot_height)
                    .child(gl_canvas)
                    .child(plot_el)
                    .into_any_element()
            } else {
                plot_el.into_any_element()
            };

            if has_y_axis || has_x_labels {
                let mut top_row = div().flex().flex_row().w_full();

                if has_y_axis {
                    let mut y_col = div()
                        .flex()
                        .flex_col()
                        .justify_between()
                        .w(y_margin)
                        .h(plot_height)
                        .pr(px(4.0));

                    for tick in y_ticks.iter().rev() {
                        let label = format_y_tick(*tick);
                        y_col = y_col.child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .text_align(TextAlign::Right)
                                .child(label),
                        );
                    }
                    top_row = top_row.child(y_col);
                }

                top_row = top_row.child(plot_wrapper);

                if has_x_labels {
                    let x_labels = axis_config.as_ref().and_then(|a| a.x_labels.clone());
                    if let Some(labels) = x_labels {
                        let max_points = data_set
                            .series
                            .iter()
                            .map(|s| s.inner.values.len())
                            .max()
                            .unwrap_or(0)
                            .max(1);
                        let mut x_row = div().flex().flex_row().w_full().h(x_margin).ml(y_margin);

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
                        container = container.child(top_row).child(x_row);
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

        // Auto-show legend for multi-series charts.
        if show_legend && data_set.series.len() > 1 {
            let mut legend_row = div()
                .flex()
                .flex_row()
                .gap(px(12.0))
                .px(px(8.0))
                .pt(px(4.0))
                .w(total_width);

            for (si, series) in data_set.series.iter().enumerate() {
                let color = Self::series_color(&data_set, global_color, si, theme);
                legend_row = legend_row.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.0))
                        .child(div().size(px(8.0)).rounded(theme.radius_full).bg(color))
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .child(series.inner.name.clone()),
                        ),
                );
            }

            let chart_el = div().flex().flex_col().child(container).child(legend_row);
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
    let mut wrapper = div().flex().flex_col().gap(px(TITLE_GAP));
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
    min: f32,
    range: f32,
    fka_points: &Option<(FocusGroup, Vec<FocusHandle>)>,
    point_prefix: &Option<SharedString>,
    total_fka_points: usize,
    series_offsets: &[usize],
    multi_series: bool,
    chart_type: ChartType,
    dwc: bool,
) -> gpui::Div {
    let n_series = data_set.series.len();
    let max_points = data_set
        .series
        .iter()
        .map(|s| s.inner.values.len())
        .max()
        .unwrap_or(0)
        .max(1);
    let slot_width = f32::from(width) / max_points as f32;

    let bar_count_per_slot = n_series.max(1);
    let total_bar_gap = BAR_GAP * (bar_count_per_slot - 1) as f32;
    let bar_width =
        ((slot_width * BAR_WIDTH_RATIO - total_bar_gap) / bar_count_per_slot as f32).max(1.0);
    let group_width = bar_width * bar_count_per_slot as f32 + total_bar_gap;
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
            let v = series.inner.values.get(slot_i).copied().unwrap_or(0.0);
            let norm = ((v - min) / range).clamp(0.0, 1.0);
            let bar_h = f32::from(height) * norm;
            let color = Chart::series_color(data_set, global_color, si, theme);
            let mut bar = div()
                .w(px(bar_width))
                .h(px(bar_h))
                .bg(color)
                .rounded(theme.radius_sm);
            if dwc {
                bar = bar.border_1().border_color(theme.text);
            }

            // Contiguous handle offset per series — works correctly when
            // series have different lengths (unlike `si * max_points + slot_i`).
            let fka_idx = series_offsets[si] + slot_i;
            let bar = match (fka_points.as_ref(), point_prefix.as_ref()) {
                (Some((group, handles)), Some(prefix))
                    if fka_idx < handles.len() && slot_i < series.inner.values.len() =>
                {
                    attach_fka(
                        bar,
                        &FkaAttachContext {
                            group,
                            handles,
                            prefix,
                            total: total_fka_points,
                            chart_type,
                            theme,
                            series_name: &series.inner.name,
                            multi_series,
                        },
                        fka_idx,
                        v,
                        window,
                    )
                }
                _ => bar.into_any_element(),
            };
            group = group.child(bar);
        }
        row = row.child(group);
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
    min: f32,
    range: f32,
    fka_points: &Option<(FocusGroup, Vec<FocusHandle>)>,
    point_prefix: &Option<SharedString>,
    total_fka_points: usize,
    series_offsets: &[usize],
    multi_series: bool,
    chart_type: ChartType,
    dwc: bool,
) -> gpui::Div {
    let max_points = data_set
        .series
        .iter()
        .map(|s| s.inner.values.len())
        .max()
        .unwrap_or(0)
        .max(1);
    let slot_width = f32::from(width) / max_points as f32;
    let point_size = MIN_POINT_SIZE.max(slot_width.min(MAX_POINT_SIZE));

    let mut row = div().flex().flex_row().items_end().w(width).h(height);

    for (slot_i, _point) in (0..max_points).enumerate() {
        let mut cell = div().w(px(slot_width)).h(height).relative();

        for (si, series) in data_set.series.iter().enumerate() {
            let v = series.inner.values.get(slot_i).copied().unwrap_or(0.0);
            let norm = ((v - min) / range).clamp(0.0, 1.0);
            let top_offset = f32::from(height) * (1.0 - norm) - point_size / 2.0;
            let color = Chart::series_color(data_set, global_color, si, theme);

            let mut dot = div()
                .absolute()
                .top(px(top_offset.max(0.0)))
                .left(px((slot_width - point_size) / 2.0))
                .size(px(point_size))
                .bg(color);

            dot = apply_marker_shape(dot, si, point_size, theme);

            if dwc {
                dot = dot.border_1().border_color(theme.text);
            }

            let fka_idx = series_offsets[si] + slot_i;
            let dot = match (fka_points.as_ref(), point_prefix.as_ref()) {
                (Some((group, handles)), Some(prefix))
                    if fka_idx < handles.len() && slot_i < series.inner.values.len() =>
                {
                    attach_fka(
                        dot,
                        &FkaAttachContext {
                            group,
                            handles,
                            prefix,
                            total: total_fka_points,
                            chart_type,
                            theme,
                            series_name: &series.inner.name,
                            multi_series,
                        },
                        fka_idx,
                        v,
                        window,
                    )
                }
                _ => dot.into_any_element(),
            };
            cell = cell.child(dot);
        }
        row = row.child(cell);
    }
    row
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
fn apply_marker_shape(
    dot: gpui::Div,
    si: usize,
    point_size: f32,
    theme: &crate::foundations::theme::TahoeTheme,
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
            .border_color(theme.text)
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
    min: f32,
    range: f32,
    fka_points: &Option<(FocusGroup, Vec<FocusHandle>)>,
    point_prefix: &Option<SharedString>,
    total_fka_points: usize,
    series_offsets: &[usize],
    multi_series: bool,
    chart_type: ChartType,
    dwc: bool,
) -> gpui::Div {
    let w_f = f32::from(width);
    let h_f = f32::from(height);

    let series_data: Vec<(Vec<f32>, Option<Vec<f32>>, Hsla)> = data_set
        .series
        .iter()
        .enumerate()
        .map(|(si, s)| {
            (
                s.inner.values.clone(),
                s.inner.range_low.clone(),
                Chart::series_color(data_set, global_color, si, theme),
            )
        })
        .collect();

    let max_points = series_data
        .iter()
        .map(|(v, _, _)| v.len())
        .max()
        .unwrap_or(0);

    let canvas_el = canvas(
        |_info, _window, _cx| {},
        move |bounds, _state, window, _cx| {
            let origin = bounds.origin;
            let bw = f32::from(bounds.size.width);
            let bh = f32::from(bounds.size.height);
            for (values, range_low, color) in series_data {
                if let Some(paint) = canvas_paint_callback(
                    chart_type, origin, bw, bh, values, range_low, min, range, color,
                ) {
                    paint(window);
                }
            }
        },
    )
    .w(width)
    .h(height);

    if let (Some((group, handles)), Some(prefix)) = (fka_points.as_ref(), point_prefix.as_ref()) {
        let mut row = div()
            .absolute()
            .top_0()
            .left_0()
            .w(width)
            .h(height)
            .flex()
            .flex_row();

        // Hoist slot_width: independent of `si`.
        let slot_width = w_f / max_points.max(1) as f32;

        for (si, series) in data_set.series.iter().enumerate() {
            for (slot_i, v) in series.inner.values.iter().enumerate() {
                // Contiguous handle offset — indexing as `si * max_points +
                // slot_i` is wrong for unequal-length series.
                let fka_idx = series_offsets[si] + slot_i;
                if fka_idx >= handles.len() {
                    break;
                }
                let mut hit = div().w(px(slot_width)).h(height).opacity(0.0);
                if dwc {
                    let point_size = MIN_POINT_SIZE.max(slot_width.min(MAX_POINT_SIZE));
                    let norm = ((v - min) / range).clamp(0.0, 1.0);
                    let top_offset = h_f * (1.0 - norm) - point_size / 2.0;
                    // Transparent ring overlay on top of the canvas mark;
                    // the shape rotates per series so ≥4-series charts stay
                    // distinguishable without colour.
                    let indicator = div()
                        .absolute()
                        .top(px(top_offset.max(0.0)))
                        .left(px((slot_width - point_size) / 2.0))
                        .size(px(point_size))
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
                        chart_type,
                        theme,
                        series_name: &series.inner.name,
                        multi_series,
                    },
                    fka_idx,
                    *v,
                    window,
                );
                row = row.child(hit);
            }
        }

        div()
            .relative()
            .w(width)
            .h(height)
            .child(canvas_el)
            .child(row)
    } else {
        div().w(width).h(height).child(canvas_el)
    }
}
