//! HIG Chart component.
//!
//! Minimal Bar and Line chart primitives rendered through GPUI `div`s.
//! Apple's Swift Charts exposes a declarative `Mark` API over axes,
//! gridlines, and series; this component covers the two most common mark
//! types (Bar, Line) which is what the HIG Charts page documents as
//! must-have coverage for macOS 26 data surfaces.
//!
//! Intentional scope for v1: single-series, rendering only. Axis labels,
//! gridlines, legends, and multi-series overlay are captured in the data
//! types so future versions can layer them on without breaking the
//! builder.
//!
//! # Accessibility
//!
//! Per HIG, each chart exposes a VoiceOver summary string through
//! [`Chart::accessibility_label`]. The default label is
//! `"{type} chart: {count} values, range {min}–{max}"` so the chart is
//! announced with actionable context even if the caller supplies no label.
//!
//! # Color independence
//!
//! Charts never convey meaning through colour alone. Bars and points are
//! rendered in `theme.accent`; multi-series support (when added) will
//! layer distinct marker shapes as well as hues to stay HIG-compliant
//! when Differentiate Without Color is active.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/charts>

use gpui::prelude::*;
use gpui::{
    App, ElementId, FocusHandle, Hsla, KeyDownEvent, Pixels, SharedString, Window, div, px,
};

use crate::foundations::accessibility::{
    AccessibilityProps, AccessibilityRole, AccessibleExt, FocusGroup, FocusGroupExt,
};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::ActiveTheme;

/// Chart mark type.
///
/// Mirrors Swift Charts' `Mark` vocabulary. v1 ships bar-based and
/// sparkline-based rendering; the remaining variants are approximated
/// with the closest available mark so the API surface is stable while we
/// wait for GPUI canvas-stroked lines and area fills.
///
/// # v1 rendering
///
/// | Variant | Rendered as              |
/// |---------|--------------------------|
/// | `Bar`   | Native bar columns       |
/// | `Area`  | Bar columns (no fill)    |
/// | `Range` | Bar columns (no range)   |
/// | `Line`  | Point sparkline          |
/// | `Point` | Point sparkline (native) |
/// | `Rule`  | Point sparkline          |
///
/// `voice_label()` still returns the caller-supplied semantic name so the
/// VoiceOver announcement stays honest about intent even when the visual
/// is a fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ChartType {
    /// Native bar columns. Full HIG coverage.
    #[default]
    Bar,
    /// Point sparkline. HIG expects a stroked polyline — falls back to
    /// points-only until GPUI canvas lands.
    Line,
    /// Area mark. Falls back to `Bar` rendering in v1.
    Area,
    /// Point sparkline. Full HIG coverage (no stroke needed).
    Point,
    /// Range mark. Falls back to `Bar` rendering in v1 (min/max endpoints
    /// not yet rendered).
    Range,
    /// Rule mark (horizontal/vertical reference line). Falls back to point
    /// sparkline in v1.
    Rule,
}

impl ChartType {
    /// The lowercase name used for VoiceOver announcements
    /// ("bar chart: …").
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
}

/// A single named data series.
#[derive(Debug, Clone)]
pub struct ChartDataSeries {
    pub name: SharedString,
    pub values: Vec<f32>,
}

impl ChartDataSeries {
    pub fn new(name: impl Into<SharedString>, values: Vec<f32>) -> Self {
        Self {
            name: name.into(),
            values,
        }
    }

    fn min_value(&self) -> f32 {
        self.values.iter().copied().fold(f32::INFINITY, f32::min)
    }

    fn max_value(&self) -> f32 {
        self.values
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max)
    }
}

/// HIG chart primitive.
#[derive(IntoElement)]
pub struct Chart {
    id: ElementId,
    chart_type: ChartType,
    series: ChartDataSeries,
    width: Pixels,
    height: Pixels,
    color: Option<Hsla>,
    accessibility_label: Option<SharedString>,
    point_focus_group: Option<FocusGroup>,
    point_focus_handles: Vec<FocusHandle>,
}

impl Chart {
    /// Create a new chart for the given series.
    ///
    /// The default id is `"chart"`; callers rendering more than one chart
    /// in the same window must override via [`Chart::id`] so GPUI's
    /// per-render-tree element-id uniqueness invariant holds.
    pub fn new(series: ChartDataSeries) -> Self {
        Self {
            id: ElementId::Name(SharedString::from("chart")),
            chart_type: ChartType::default(),
            series,
            width: px(240.0),
            height: px(120.0),
            color: None,
            accessibility_label: None,
            point_focus_group: None,
            point_focus_handles: Vec::new(),
        }
    }

    /// Override the chart's root element id. Used as the prefix for every
    /// per-data-point element id so two charts in the same window do not
    /// collide. Without this, a second chart reusing the default `"chart"`
    /// prefix would clash with the first in GPUI's per-render-tree id map.
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

    /// Attach a host-owned [`FocusGroup`] for per-data-point arrow-nav and
    /// Tab-reachability under macOS Full Keyboard Access. Paired with
    /// [`Chart::point_focus_handles`]. Use [`FocusGroup::open`] so Tab
    /// still exits the chart naturally; Left/Up and Right/Down move focus
    /// along the axis via `group.focus_next` / `focus_previous`, and
    /// Home/End jump to the first and last data point.
    pub fn point_focus_group(mut self, group: FocusGroup) -> Self {
        self.point_focus_group = Some(group);
        self
    }

    /// Per-data-point [`FocusHandle`]s. The chart expects one handle per
    /// value in the series, in series order. Host-owned (stateless
    /// components cannot keep them across renders).
    pub fn point_focus_handles(mut self, handles: Vec<FocusHandle>) -> Self {
        self.point_focus_handles = handles;
        self
    }

    /// Build the default VoiceOver label per HIG guidance.
    fn default_accessibility_label(&self) -> String {
        let count = self.series.values.len();
        if count == 0 {
            return format!(
                "{} chart: {}, no values",
                self.chart_type.voice_label(),
                self.series.name
            );
        }
        let min = self.series.min_value();
        let max = self.series.max_value();
        format!(
            "{} chart: {}, {} values, range {:.2} to {:.2}",
            self.chart_type.voice_label(),
            self.series.name,
            count,
            min,
            max
        )
    }
}

impl RenderOnce for Chart {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let bar_color = self.color.unwrap_or(theme.accent);

        let width = self.width;
        let height = self.height;

        let min = self.series.min_value().min(0.0); // anchor at zero for bar charts so small values stay visible.
        let max = self.series.max_value().max(1e-3);
        let range = (max - min).max(1e-3);

        let chart_type = self.chart_type;
        let a11y_label: SharedString = match self.accessibility_label.take() {
            Some(label) => label,
            None => SharedString::from(self.default_accessibility_label()),
        };
        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::Group);

        // Move the values + root id out of `self` so the render path
        // doesn't clone the full `Vec<f32>` on every frame.
        let values = self.series.values;
        let root_id = self.id;

        // FKA: only attach per-point focus when the flag is set AND the
        // host supplied both a FocusGroup and exactly one handle per value.
        // Tracking handle-count strictly avoids half-registered state that
        // would leak stale tab indices between renders.
        let fka_points = FocusGroup::bind_if_fka(
            theme.full_keyboard_access(),
            self.point_focus_group,
            self.point_focus_handles,
            values.len(),
        );

        // Per-point element-id prefix: embeds the chart's root id so two
        // charts in the same window don't collide in GPUI's id map. Built
        // once per render; each bar/dot does a SharedString Arc bump.
        let point_prefix: Option<SharedString> = fka_points
            .as_ref()
            .map(|_| SharedString::from(format!("{root_id}-point")));
        let total_points = values.len();

        // Build the inner row of marks first so the container styling
        // can be applied (or not) to a single child via `.child(row)`.
        let inner: Option<gpui::Div> = if values.is_empty() {
            None
        } else {
            Some(match chart_type {
                ChartType::Bar | ChartType::Area | ChartType::Range => {
                    let count = values.len().max(1);
                    let slot_width = f32::from(width) / count as f32;
                    let bar_width = (slot_width * 0.7).max(1.0);
                    let gap = (slot_width - bar_width) / 2.0;

                    let mut row = div()
                        .flex()
                        .flex_row()
                        .items_end()
                        .w(width)
                        .h(height)
                        .px(px(gap))
                        .gap(px((slot_width - bar_width).max(0.0)));

                    for (i, v) in values.iter().enumerate() {
                        let norm = ((v - min) / range).clamp(0.0, 1.0);
                        let bar_h = f32::from(height) * norm;
                        let bar = div()
                            .w(px(bar_width))
                            .h(px(bar_h))
                            .bg(bar_color)
                            .rounded(theme.radius_sm);
                        let bar = match (fka_points.as_ref(), point_prefix.as_ref()) {
                            (Some((group, handles)), Some(prefix)) => attach_fka(
                                bar,
                                group,
                                handles,
                                prefix,
                                i,
                                total_points,
                                *v,
                                chart_type,
                                theme,
                                window,
                            ),
                            _ => bar.into_any_element(),
                        };
                        row = row.child(bar);
                    }
                    row
                }
                ChartType::Line | ChartType::Point | ChartType::Rule => {
                    // Render a sparkline: point markers placed in a flex
                    // row whose vertical alignment encodes the value. A
                    // true connecting stroke needs canvas rendering (GPUI
                    // `canvas`) which is out of scope for v1.
                    let count = values.len().max(1);
                    let slot_width = f32::from(width) / count as f32;
                    let point_size = 4.0_f32.max(slot_width.min(10.0));

                    let mut row = div().flex().flex_row().items_end().w(width).h(height);

                    for (i, v) in values.iter().enumerate() {
                        let norm = ((v - min) / range).clamp(0.0, 1.0);
                        let top_offset = f32::from(height) * (1.0 - norm) - point_size / 2.0;
                        let dot = div()
                            .absolute()
                            .top(px(top_offset.max(0.0)))
                            .left(px((slot_width - point_size) / 2.0))
                            .size(px(point_size))
                            .rounded(theme.radius_full)
                            .bg(bar_color);
                        let dot = match (fka_points.as_ref(), point_prefix.as_ref()) {
                            (Some((group, handles)), Some(prefix)) => attach_fka(
                                dot,
                                group,
                                handles,
                                prefix,
                                i,
                                total_points,
                                *v,
                                chart_type,
                                theme,
                                window,
                            ),
                            _ => dot.into_any_element(),
                        };
                        let cell = div().w(px(slot_width)).h(height).relative().child(dot);
                        row = row.child(cell);
                    }
                    row
                }
            })
        };

        // Container: base styling. Under FKA the per-bar/per-dot handles
        // are the Tab stops — the container itself deliberately stays
        // non-focusable so Tab lands on the first data point rather than
        // a wrapper.
        let mut container = div()
            .w(width)
            .h(height)
            .bg(theme.surface)
            .rounded(theme.radius_md)
            .border_1()
            .border_color(theme.border)
            .overflow_hidden()
            .with_accessibility(&a11y_props);
        if let Some(inner) = inner {
            container = container.child(inner);
        }
        container
    }
}

/// Wire a bar or point div up for Full Keyboard Access: per-value element
/// id, focus-group registration, per-value VoiceOver label, focus ring,
/// and arrow/Home/End/activation key handling.
///
/// Extracted because the Bar and Line branches of the chart share every
/// piece of this wiring; inlining duplicated ~40 lines in two places.
#[allow(clippy::too_many_arguments)]
fn attach_fka(
    el: gpui::Div,
    group: &FocusGroup,
    handles: &[FocusHandle],
    prefix: &SharedString,
    index: usize,
    total: usize,
    value: f32,
    chart_type: ChartType,
    theme: &crate::foundations::theme::TahoeTheme,
    window: &Window,
) -> gpui::AnyElement {
    let is_focused = handles[index].is_focused(window);
    let a11y = AccessibilityProps::new()
        .label(SharedString::from(format!(
            "{}: {} of {}, {:.2}",
            chart_type.voice_label(),
            index + 1,
            total,
            value
        )))
        .role(AccessibilityRole::Button);
    let group_for_keys = group.clone();
    let el = el
        .id((prefix.clone(), index))
        .focus_group(group, &handles[index])
        .with_accessibility(&a11y)
        .on_key_down(
            move |ev: &KeyDownEvent, window, cx| match ev.keystroke.key.as_str() {
                "left" | "up" => {
                    group_for_keys.focus_previous(window, cx);
                    cx.stop_propagation();
                }
                "right" | "down" => {
                    group_for_keys.focus_next(window, cx);
                    cx.stop_propagation();
                }
                "home" => {
                    group_for_keys.focus_first(window, cx);
                    cx.stop_propagation();
                }
                "end" => {
                    group_for_keys.focus_last(window, cx);
                    cx.stop_propagation();
                }
                _ => {}
            },
        );
    apply_focus_ring(el, theme, is_focused, &[]).into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{Chart, ChartDataSeries, ChartType};
    use core::prelude::v1::test;
    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext, Window, hsla, px};

    use crate::foundations::accessibility::{AccessibilityMode, FocusGroup};
    use crate::foundations::theme::TahoeTheme;
    use crate::test_helpers::helpers::setup_test_window;

    fn series() -> ChartDataSeries {
        ChartDataSeries::new("Sales", vec![10.0, 20.0, 15.0, 30.0, 25.0])
    }

    #[test]
    fn chart_default_type_is_bar() {
        let chart = Chart::new(series());
        assert_eq!(chart.chart_type, ChartType::Bar);
    }

    #[test]
    fn chart_builder_sets_type() {
        let chart = Chart::new(series()).chart_type(ChartType::Line);
        assert_eq!(chart.chart_type, ChartType::Line);
    }

    #[test]
    fn chart_builder_sets_size() {
        let chart = Chart::new(series()).size(px(300.0), px(160.0));
        assert_eq!(chart.width, px(300.0));
        assert_eq!(chart.height, px(160.0));
    }

    #[test]
    fn chart_builder_sets_color() {
        let c = hsla(0.3, 1.0, 0.5, 1.0);
        let chart = Chart::new(series()).color(c);
        assert_eq!(chart.color, Some(c));
    }

    #[test]
    fn chart_builder_sets_accessibility_label() {
        let chart = Chart::new(series()).accessibility_label("Quarterly sales");
        assert_eq!(
            chart.accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("Quarterly sales")
        );
    }

    #[test]
    fn chart_voice_label_covers_all_mark_types() {
        assert_eq!(ChartType::Bar.voice_label(), "bar");
        assert_eq!(ChartType::Line.voice_label(), "line");
        assert_eq!(ChartType::Area.voice_label(), "area");
        assert_eq!(ChartType::Point.voice_label(), "point");
        assert_eq!(ChartType::Range.voice_label(), "range");
        assert_eq!(ChartType::Rule.voice_label(), "rule");
    }

    #[test]
    fn default_accessibility_label_includes_type_name_count_range() {
        let chart = Chart::new(series()).chart_type(ChartType::Bar);
        let label = chart.default_accessibility_label();
        assert!(label.starts_with("bar chart:"), "got {label:?}");
        assert!(label.contains("Sales"));
        assert!(label.contains("5 values"));
        assert!(label.contains("10.00"));
        assert!(label.contains("30.00"));
    }

    #[test]
    fn default_accessibility_label_handles_empty_series() {
        let chart = Chart::new(ChartDataSeries::new("Empty", vec![]));
        let label = chart.default_accessibility_label();
        assert!(label.contains("no values"));
    }

    #[test]
    fn data_series_min_max() {
        let s = series();
        assert!((s.min_value() - 10.0).abs() < f32::EPSILON);
        assert!((s.max_value() - 30.0).abs() < f32::EPSILON);
    }

    // ─── HIG: Full Keyboard Access ───────────────────────────────────

    struct ChartFkaHarness {
        handles: Vec<FocusHandle>,
        group: FocusGroup,
        chart_type: ChartType,
    }

    impl ChartFkaHarness {
        fn new(cx: &mut Context<Self>, count: usize, chart_type: ChartType) -> Self {
            Self {
                handles: (0..count).map(|_| cx.focus_handle()).collect(),
                group: FocusGroup::open(),
                chart_type,
            }
        }
    }

    impl Render for ChartFkaHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(series())
                .chart_type(self.chart_type)
                .point_focus_group(self.group.clone())
                .point_focus_handles(self.handles.clone())
        }
    }

    #[gpui::test]
    async fn fka_off_does_not_register_point_handles(cx: &mut TestAppContext) {
        let (host, _cx) = setup_test_window(cx, |_window, cx| {
            ChartFkaHarness::new(cx, 5, ChartType::Bar)
        });
        host.update(cx, |host, _cx| {
            assert!(host.group.is_empty());
        });
    }

    #[gpui::test]
    async fn fka_on_registers_one_focus_per_point(cx: &mut TestAppContext) {
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            ChartFkaHarness::new(cx, 5, ChartType::Bar)
        });
        host.update(vcx, |host, _cx| {
            assert_eq!(host.group.len(), 5);
        });
    }

    #[gpui::test]
    async fn fka_on_registers_one_focus_per_point_for_line(cx: &mut TestAppContext) {
        // Pins the Line/Point/Rule branch against the Bar branch — both
        // must wire per-data-point focus handles under FKA.
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            ChartFkaHarness::new(cx, 5, ChartType::Line)
        });
        host.update(vcx, |host, _cx| {
            assert_eq!(host.group.len(), 5);
        });
    }

    #[gpui::test]
    async fn fka_on_preserves_registration_order(cx: &mut TestAppContext) {
        // Arrow-nav correctness is the FocusGroup's responsibility and
        // is covered by `focus_group::tests`. What the chart must
        // guarantee is that handles are registered in series-order so
        // `focus_next`/`focus_previous` walk left-to-right along the
        // x-axis.
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            ChartFkaHarness::new(cx, 3, ChartType::Bar)
        });
        host.update(vcx, |host, _cx| {
            for (i, handle) in host.handles.iter().enumerate() {
                assert_eq!(host.group.register(handle), i);
            }
        });
    }

    #[gpui::test]
    async fn fka_on_mismatched_handle_count_skips_registration(cx: &mut TestAppContext) {
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            ChartFkaHarness {
                group: FocusGroup::open(),
                handles: vec![cx.focus_handle(), cx.focus_handle()], // 2 for 5
                chart_type: ChartType::Bar,
            }
        });
        host.update(vcx, |host, _cx| {
            assert!(host.group.is_empty());
        });
    }

    #[gpui::test]
    async fn fka_on_focus_next_advances_along_axis(cx: &mut TestAppContext) {
        // With focus on handle[0], focus_next must land on handle[1] —
        // proves the registered order matches the x-axis series order
        // end-to-end through the FocusGroup's programmatic navigation.
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            ChartFkaHarness::new(cx, 5, ChartType::Bar)
        });
        host.update_in(vcx, |host, window, cx| {
            host.handles[0].focus(window, cx);
            host.group.focus_next(window, cx);
            assert!(host.handles[1].is_focused(window));
        });
    }

    #[gpui::test]
    async fn fka_on_focus_previous_retreats_along_axis(cx: &mut TestAppContext) {
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            ChartFkaHarness::new(cx, 5, ChartType::Bar)
        });
        host.update_in(vcx, |host, window, cx| {
            host.handles[2].focus(window, cx);
            host.group.focus_previous(window, cx);
            assert!(host.handles[1].is_focused(window));
        });
    }

    #[gpui::test]
    async fn fka_on_focus_first_and_last_jump_to_edges(cx: &mut TestAppContext) {
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            ChartFkaHarness::new(cx, 5, ChartType::Bar)
        });
        host.update_in(vcx, |host, window, cx| {
            host.handles[2].focus(window, cx);
            host.group.focus_last(window, cx);
            assert!(
                host.handles[4].is_focused(window),
                "focus_last lands on final registered handle (End key)"
            );
            host.group.focus_first(window, cx);
            assert!(
                host.handles[0].is_focused(window),
                "focus_first lands on first registered handle (Home key)"
            );
        });
    }
}
