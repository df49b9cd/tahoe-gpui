//! Stateful chart with interactive hover tooltip.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    Context, ElementId, FocusHandle, Hsla, IntoElement, KeyDownEvent, MouseMoveEvent, Pixels,
    ScrollDelta, ScrollWheelEvent, SharedString, Task, Window, canvas, div, px,
};

use std::sync::Arc;

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};

use super::animation::{DATA_TRANSITION_DURATION, interpolate_data_set};
use super::audio_graph::ChartDescriptor;
use super::render::series_color;
use super::scroll::ChartScrollConfig;
use super::types::{AxisConfig, ChartDataSet, ChartType, GridlineConfig, PlottableValue};

/// Selection push-to-parent hook.
///
/// Mirrors Swift Charts' `.chartXSelection(value:)` binding. Installed via
/// [`ChartView::selection_binding`] and fired whenever the effective
/// hover/focus slot changes. The callback receives the full [`SelectedPoint`]
/// for the primary series (or `None` when the pointer and keyboard focus
/// are both cleared), so host apps can drive external read-outs, filter
/// sibling views, or trigger navigation.
pub type SelectionBinding =
    Rc<dyn Fn(&mut ChartView, Option<SelectedPoint>, &mut Context<ChartView>) + 'static>;

/// Payload delivered to [`SelectionBinding`] callbacks.
///
/// Mirrors Swift Charts' `ChartProxy.value(atX:)` result: the primary-series
/// name, the X value of the hovered slot, and the Y value of that series at
/// the slot. Multi-series charts only surface the first series so the
/// `Option<_>` signature stays the same as Swift Charts' upstream; parents
/// can still look up sibling series by the reported X value.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedPoint {
    /// The `ChartDataSeries::name` of the series whose point is surfaced.
    pub series_name: SharedString,
    /// X value of the selected slot.
    pub x: PlottableValue,
    /// Y value of the selected slot on the primary series.
    pub y: PlottableValue,
}

/// Interactive chart view with hover tooltips and crosshair.
///
/// Wraps the stateless [`super::Chart`] with a transparent mouse-tracking
/// overlay that computes the nearest data-point index and renders a
/// vertical crosshair line plus value tooltip.
pub struct ChartView {
    id: SharedString,
    chart_type: ChartType,
    data_set: ChartDataSet,
    width: Pixels,
    height: Pixels,
    global_color: Option<Hsla>,
    axis: Option<AxisConfig>,
    gridlines: Option<GridlineConfig>,
    focus_handle: FocusHandle,
    /// Pointer-driven hover index. Tracks the mouse and clears on
    /// `on_hover(false)`; kept separate from `focus_index` so moving the
    /// pointer off the chart doesn't erase a keyboard-selected slot (and
    /// vice versa).
    pointer_index: Option<usize>,
    /// Keyboard-driven focus index. Advances on arrow/Home/End and clears
    /// on Escape.
    focus_index: Option<usize>,
    /// Wrapper's left edge in window coordinates, captured during paint
    /// so `on_mouse_move` can translate `event.position.x` (window space)
    /// into the wrapper-local `x` that `compute_hover_index` expects.
    /// Updated from the crosshair canvas paint callback.
    wrapper_origin_x: Rc<Cell<f32>>,
    /// Optional push-to-parent selection callback. Fires whenever the
    /// effective hover/focus slot changes; the payload is `None` when both
    /// pointer and keyboard focus are cleared.
    selection_binding: Option<SelectionBinding>,
    /// Optional scroll / zoom configuration. When set with a narrower
    /// `x_visible_domain` than the data, the inner [`super::Chart`] renders
    /// only the visible slice and scroll-wheel input advances the visible
    /// window along the X axis.
    scroll: Option<ChartScrollConfig>,
    /// Optional Audio Graphs descriptor. Threaded through to the inner
    /// [`super::Chart`] at render so VoiceOver's label picks up the
    /// descriptor's `summary`, and reachable from hosts via
    /// [`ChartView::play_audio_graph`] for sonification triggers.
    audio_graph: Option<Arc<ChartDescriptor>>,
    /// Stash of the previous data set, held for the duration of an
    /// active [`ChartView::set_data`] tween. `None` outside a
    /// transition.
    previous_data_set: Option<ChartDataSet>,
    /// Wall-clock start of the active transition. Paired with
    /// [`DATA_TRANSITION_DURATION`] to compute per-frame progress.
    transition_started_at: Option<Instant>,
    /// Pending per-frame redraw ticker. Dropping the `Task` cancels it,
    /// so calling [`ChartView::set_data`] again mid-transition supersedes
    /// the prior tween without leaking a background loop.
    transition_task: Option<Task<()>>,
}

impl ChartView {
    /// Create a new interactive chart view for the given series.
    ///
    /// The returned view is an `Entity<ChartView>`-capable struct: wrap it in
    /// `cx.new(|cx| ChartView::new(cx, series))` to obtain the GPUI entity
    /// that owns the hover/focus state. Defaults mirror [`super::Chart`]:
    /// `Bar` marks at `320×180`, no axis, no gridlines, palette-assigned colours.
    pub fn new(cx: &mut Context<Self>, data_set: impl Into<ChartDataSet>) -> Self {
        Self {
            id: SharedString::from("chart-view"),
            chart_type: ChartType::default(),
            data_set: data_set.into(),
            width: px(320.0),
            height: px(180.0),
            global_color: None,
            axis: None,
            gridlines: None,
            focus_handle: cx.focus_handle(),
            pointer_index: None,
            focus_index: None,
            wrapper_origin_x: Rc::new(Cell::new(0.0)),
            selection_binding: None,
            scroll: None,
            audio_graph: None,
            previous_data_set: None,
            transition_started_at: None,
            transition_task: None,
        }
    }

    /// Override the chart's root element id.
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = id.into();
        self
    }

    /// Set the mark type rendered by the inner [`super::Chart`].
    pub fn chart_type(mut self, chart_type: ChartType) -> Self {
        self.chart_type = chart_type;
        self
    }

    /// Set the overall wrapper size. Plot area is this minus any axis margins.
    pub fn size(mut self, width: Pixels, height: Pixels) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Override the single-series mark colour. Multi-series charts auto-
    /// assign palette colours; this value only applies to series index 0.
    pub fn color(mut self, color: Hsla) -> Self {
        self.global_color = Some(color);
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

    /// Configure a scroll / zoom window over the X axis.
    ///
    /// Mirrors Swift Charts' `.chartXVisibleDomain(length:)` +
    /// `.chartScrollPosition(initialX:)` pair. The inner [`super::Chart`]
    /// renders only points whose X falls inside the effective window, and
    /// scroll-wheel input advances the window by ~10% of its width per
    /// line-tick (equivalent pixel math for trackpads).
    ///
    /// Only numeric `PlottableValue::Number` domains are honoured today.
    pub fn scroll(mut self, config: ChartScrollConfig) -> Self {
        self.scroll = Some(config);
        self
    }

    /// Attach an [`ChartDescriptor`] for VoiceOver + Audio Graphs.
    ///
    /// Mirrors Apple's `.accessibilityChartDescriptor(_:)` modifier. The
    /// descriptor's summary flows into the inner [`super::Chart`]'s
    /// VoiceOver label and the sonification surface is reachable via
    /// [`ChartView::play_audio_graph`].
    pub fn audio_graph(mut self, descriptor: ChartDescriptor) -> Self {
        self.audio_graph = Some(Arc::new(descriptor));
        self
    }

    /// Trigger sonification of the attached [`ChartDescriptor`].
    ///
    /// HIG's *Charting data* page recommends wiring this to VoiceOver's
    /// `VO + Shift + S` chord; since that chord is owned by the system-
    /// level VoiceOver, hosts typically expose this through a dedicated
    /// app-level shortcut or an accessibility button. Returns `false`
    /// when no descriptor has been attached so hosts can fall back to
    /// another presentation.
    pub fn play_audio_graph(&self) -> bool {
        match self.audio_graph.as_ref() {
            Some(desc) => {
                desc.play_sonification();
                true
            }
            None => false,
        }
    }

    /// Replace the underlying data set with a tweened transition.
    ///
    /// Mirrors Swift Charts' behaviour of animating between old and new
    /// data when the source changes — numeric Y values lerp, while Date
    /// and Category Y values cross-fade at the midpoint. Runs across the
    /// HIG medium ramp (~300 ms, matching SwiftUI's implicit `.spring`).
    ///
    /// When the active theme has `REDUCE_MOTION` set, the chart snaps
    /// to `data` immediately and no frame-tick task is spawned, matching
    /// HIG: "replace large, dramatic transitions with subtle
    /// cross-fades."
    ///
    /// Calling `set_data` mid-transition cancels the prior tween by
    /// dropping its task, stashes the current interpolated snapshot as
    /// the new previous state, and starts a fresh ~300 ms tween toward
    /// the latest target.
    pub fn set_data(&mut self, data: impl Into<ChartDataSet>, cx: &mut Context<Self>) {
        let next = data.into();
        if cx.theme().accessibility_mode.reduce_motion() {
            self.data_set = next;
            self.previous_data_set = None;
            self.transition_started_at = None;
            self.transition_task = None;
            cx.notify();
            return;
        }

        // Freeze the currently-rendered snapshot as the new previous
        // state so a mid-flight re-trigger (set_data → set_data before
        // the first tween finishes) starts from wherever the chart is
        // visually, not from the original source.
        let frozen = self.current_render_data_set();
        self.previous_data_set = Some(frozen);
        self.data_set = next;
        self.transition_started_at = Some(Instant::now());
        cx.notify();

        // Drive re-renders at ~60 Hz until the tween completes. The
        // tick loop clears `previous_data_set` on the final frame so
        // `render` falls back to the plain data path once the chart
        // has settled.
        let frame_interval = Duration::from_millis(16);
        self.transition_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(frame_interval).await;
                let finished = this
                    .update(cx, |this, cx| {
                        let done = this
                            .transition_started_at
                            .is_some_and(|start| start.elapsed() >= DATA_TRANSITION_DURATION);
                        if done {
                            this.previous_data_set = None;
                            this.transition_started_at = None;
                        }
                        cx.notify();
                        done
                    })
                    .unwrap_or(true);
                if finished {
                    break;
                }
            }
        }));
    }

    /// Compute the `[0.0, 1.0]` progress of the active transition, or
    /// `None` when no tween is running.
    fn transition_progress(&self) -> Option<f32> {
        let start = self.transition_started_at?;
        let elapsed = start.elapsed();
        if elapsed >= DATA_TRANSITION_DURATION {
            return Some(1.0);
        }
        Some(elapsed.as_secs_f32() / DATA_TRANSITION_DURATION.as_secs_f32())
    }

    /// Resolve the data set currently visible on screen, blending the
    /// stashed previous data set with `self.data_set` when a tween is
    /// active. Cheap clone of `self.data_set` when no transition is
    /// running.
    fn current_render_data_set(&self) -> ChartDataSet {
        match (&self.previous_data_set, self.transition_progress()) {
            (Some(prev), Some(progress)) if progress < 1.0 => {
                interpolate_data_set(prev, &self.data_set, progress)
            }
            _ => self.data_set.clone(),
        }
    }

    /// Install a push-to-parent selection callback.
    ///
    /// Mirrors Swift Charts' `.chartXSelection(value:)` binding. The
    /// callback fires whenever the effective hover/focus slot changes —
    /// mouse move, mouse leave, arrow key, Home/End, or Escape — with
    /// `None` when both pointer and keyboard focus are cleared. The
    /// payload carries the primary series' point at the selected slot
    /// (see [`SelectedPoint`]), so host apps can drive external
    /// read-outs, filter sibling views, or trigger navigation without
    /// maintaining their own hover state.
    pub fn selection_binding<F>(mut self, on_change: F) -> Self
    where
        F: Fn(&mut ChartView, Option<SelectedPoint>, &mut Context<ChartView>) + 'static,
    {
        self.selection_binding = Some(Rc::new(on_change));
        self
    }

    /// Resolve the effective hover slot. Pointer wins over keyboard focus
    /// when both are live.
    fn effective_index(&self) -> Option<usize> {
        self.pointer_index.or(self.focus_index)
    }

    /// Build a [`SelectedPoint`] for the primary (first) series at `idx`.
    /// Returns `None` when the data set is empty or the primary series
    /// doesn't have a point at that slot (ragged multi-series).
    fn build_selected_point(&self, idx: usize) -> Option<SelectedPoint> {
        let series = self.data_set.series.first()?;
        let point = series.inner.points.get(idx)?;
        Some(SelectedPoint {
            series_name: series.inner.name.clone(),
            x: point.x.clone(),
            y: point.y.clone(),
        })
    }

    /// Fire the selection binding if the effective index changed from
    /// `previous`. Skipped when no binding is installed or the slot is
    /// unchanged — this keeps pixel-frequency mouse moves that stay
    /// within a slot from notifying the host every frame.
    fn fire_selection_if_changed(&mut self, previous: Option<usize>, cx: &mut Context<Self>) {
        let next = self.effective_index();
        if next == previous {
            return;
        }
        // Clone the Rc so the closure can call back into `self` safely.
        if let Some(cb) = self.selection_binding.clone() {
            let selection = next.and_then(|idx| self.build_selected_point(idx));
            cb(self, selection, cx);
        }
    }

    /// Advance the scroll position by `delta` data units along the X axis.
    ///
    /// Used by the scroll-wheel listener. Clamps the new position to
    /// `[d_lo, d_hi - width]` so the window never slides past the data.
    /// A negative `delta` scrolls backward.
    fn advance_scroll_position(&mut self, delta: f64) -> bool {
        let Some(cfg) = self.scroll.as_ref() else {
            return false;
        };
        // Visible window width — only active when a numeric visible-domain
        // was supplied. Scrolling without a domain is a no-op.
        let (win_lo, win_hi) = match cfg.x_visible_domain.as_ref() {
            Some((lo, hi)) => match (lo.as_number(), hi.as_number()) {
                (Some(l), Some(h)) => (l, h),
                _ => return false,
            },
            None => return false,
        };
        let width = (win_hi - win_lo).max(0.0);
        if width <= 0.0 {
            return false;
        }

        // Resolve the full data extent so we can clamp. No numeric X in
        // the data → no scroll.
        let mut d_lo = f64::INFINITY;
        let mut d_hi = f64::NEG_INFINITY;
        for series in self.data_set.series.iter() {
            for p in series.inner.points.iter() {
                if let Some(xv) = p.x.as_number() {
                    d_lo = d_lo.min(xv);
                    d_hi = d_hi.max(xv);
                }
            }
        }
        if !d_lo.is_finite() || !d_hi.is_finite() {
            return false;
        }

        let current = cfg
            .x_scroll_position
            .as_ref()
            .and_then(|v| v.as_number())
            .unwrap_or(win_lo);
        let upper = (d_hi - width).max(d_lo);
        let next = (current + delta).clamp(d_lo, upper);
        if (next - current).abs() < f64::EPSILON {
            return false;
        }

        // Replace the scroll config with the advanced position; leaves
        // `x_visible_domain` and `y_scrollable` untouched.
        let new_cfg = ChartScrollConfig {
            x_visible_domain: cfg.x_visible_domain.clone(),
            x_scroll_position: Some(PlottableValue::Number(next)),
            y_scrollable: cfg.y_scrollable,
        };
        self.scroll = Some(new_cfg);
        true
    }

    fn max_points(&self) -> usize {
        self.data_set
            .series
            .iter()
            .map(|s| s.inner.points.len())
            .max()
            .unwrap_or(0)
    }

    /// Horizontal inset of the plot area from the wrapper's left edge.
    ///
    /// Matches `Chart::render`'s Y-label column so hover-x maps to the
    /// correct data-point slot when an axis is configured. The Y-label
    /// column width is theme-derived (`control_height(Mini) * 2.5`) so
    /// ChartView and Chart stay in step when the platform changes.
    fn y_margin(&self, theme: &crate::foundations::theme::TahoeTheme) -> f32 {
        if self
            .axis
            .as_ref()
            .is_some_and(|a| a.y_tick_count > 0 || a.y_ticks.is_some())
        {
            AxisConfig::y_label_width(theme)
        } else {
            0.0
        }
    }

    /// Padding applied by `Chart::render` around the plot area so data
    /// marks don't land in the rounded-corner clip region. Mirrored here
    /// so the crosshair and hover-index computations stay aligned with
    /// where the plot actually paints.
    fn plot_inset(&self, theme: &crate::foundations::theme::TahoeTheme) -> f32 {
        f32::from(theme.radius_md)
    }

    /// Height of the X-axis label row at the bottom of the wrapper.
    ///
    /// Returns 0 when no `x_labels` are configured, matching the layout
    /// decision in `Chart::render`. The crosshair must subtract this from
    /// the plot-area bottom so the vertical line stops where data stops,
    /// not inside the category-label row.
    fn x_margin(&self, theme: &crate::foundations::theme::TahoeTheme) -> f32 {
        if self.axis.as_ref().is_some_and(|a| a.x_labels.is_some()) {
            AxisConfig::x_label_height(theme)
        } else {
            0.0
        }
    }
}

impl Render for ChartView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // Blend `previous_data_set` with `self.data_set` while a Phase 12
        // tween is running; otherwise this is a ref-count clone.
        let render_data = self.current_render_data_set();
        let mut chart = super::Chart::new(render_data.clone())
            .id(self.id.clone())
            .chart_type(self.chart_type)
            .size(self.width, self.height);
        if let Some(color) = self.global_color {
            chart = chart.color(color);
        }
        if let Some(axis) = self.axis.clone() {
            chart = chart.axis(axis);
        }
        if let Some(gl) = self.gridlines.clone() {
            chart = chart.gridlines(gl);
        }
        if let Some(scroll) = self.scroll.clone() {
            chart = chart.scroll(scroll);
        }
        if let Some(desc) = self.audio_graph.clone() {
            chart = chart.audio_graph_arc(desc);
        }

        // Pointer wins when both are live (the user is actively moving the
        // mouse), otherwise fall back to the last keyboard-selected slot.
        let hover_index = self.pointer_index.or(self.focus_index);
        let global_color = self.global_color;
        let width = self.width;
        let height = self.height;
        let max_pts = self.max_points();
        let y_margin = self.y_margin(theme);
        let x_margin = self.x_margin(theme);
        let plot_inset = self.plot_inset(theme);
        let chart_type = self.chart_type;

        let crosshair_color = theme.text_muted;
        let origin_tracker = self.wrapper_origin_x.clone();
        let crosshair = canvas(
            |_info, _window, _cx| {},
            move |bounds, _state, window, _cx| {
                // `MouseMoveEvent.position` arrives in window coordinates;
                // cache the wrapper's window-space left edge each paint so
                // `on_mouse_move` can translate back to local-x.
                origin_tracker.set(f32::from(bounds.origin.x));

                let Some(idx) = hover_index else { return };
                if max_pts == 0 {
                    return;
                }
                // Crosshair x lives inside the plot area (wrapper width
                // minus the plot inset on both sides and the Y-label
                // column). Line/Area/Range paint marks at
                // `plot_w * i / (n-1)`; Bar/Point/Rule paint at slot
                // centres `slot_w * (i + 0.5)`. Branching here keeps the
                // crosshair over the actual data point.
                let plot_w = (f32::from(bounds.size.width) - 2.0 * plot_inset - y_margin).max(0.0);
                let x_offset = crosshair_x_offset(chart_type, plot_w, idx, max_pts);
                let x = bounds.origin.x + gpui::px(plot_inset + y_margin + x_offset);

                // Keep the crosshair vertically inside the plot area so it
                // matches where data paints (Chart insets by `plot_inset`
                // on top and bottom, and reserves `x_margin` at the bottom
                // for the X-axis category labels when configured).
                let top = bounds.origin.y + gpui::px(plot_inset);
                let bottom = bounds.origin.y
                    + gpui::px((f32::from(bounds.size.height) - plot_inset - x_margin).max(0.0));
                let mut pb = gpui::PathBuilder::stroke(gpui::px(1.0));
                pb.move_to(gpui::point(x, top));
                pb.line_to(gpui::point(x, bottom));
                if let Ok(path) = pb.build() {
                    window.paint_path(path, crosshair_color);
                }
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .w(width)
        .h(height);

        let tooltip_el = if let Some(idx) = hover_index {
            // Single pass: format each row label once and keep the colour
            // for the legend swatch. The per-series `format!` calls used
            // to happen twice (once for the VoiceOver summary and once per
            // child row) on every hover tick — consolidating saves the
            // duplicate allocations.
            let items: Vec<(String, Hsla)> = render_data
                .series
                .iter()
                .enumerate()
                .map(|(si, series)| {
                    let value = series
                        .inner
                        .points
                        .get(idx)
                        .and_then(|p| p.y.as_number_f32());
                    let color = series_color(&render_data, global_color, si, theme);
                    let label = match value {
                        Some(v) => format!("{}: {v:.1}", series.inner.name),
                        None => format!("{}: —", series.inner.name),
                    };
                    (label, color)
                })
                .collect();

            // Single concatenated VoiceOver label so the tooltip's Tooltip
            // role carries meaningful content. Without this the hover
            // value is invisible to assistive tech.
            let tooltip_label: SharedString = SharedString::from(
                items
                    .iter()
                    .map(|(l, _)| l.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            let a11y = AccessibilityProps::new()
                .role(AccessibilityRole::Tooltip)
                .label(tooltip_label);

            // Auto-flip: park the tooltip on the opposite side of the
            // hovered data point so it doesn't cover the mark the user is
            // inspecting. Integer midpoint (< max / 2) is exact for both
            // even and odd series lengths.
            let place_right = max_pts > 0 && idx < max_pts / 2;
            let mut tooltip_div = div()
                .absolute()
                .top(px(4.0))
                .bg(theme.surface)
                .rounded(theme.radius_sm)
                .border_1()
                .border_color(theme.border)
                .p(px(6.0))
                .gap(px(2.0))
                .flex()
                .flex_col()
                .with_accessibility(&a11y);
            tooltip_div = if place_right {
                tooltip_div.right(px(4.0))
            } else {
                tooltip_div.left(px(4.0))
            };

            for (label, color) in items {
                tooltip_div = tooltip_div.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.0))
                        .child(div().size(px(6.0)).rounded(theme.radius_full).bg(color))
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text)
                                .child(label),
                        ),
                );
            }
            Some(tooltip_div.into_any_element())
        } else {
            None
        };

        // y_margin and plot_inset are theme-derived and theme isn't
        // reachable from listener closures — snapshot them each render
        // and move the values into each closure.
        let captured_width = f32::from(width);
        let captured_y_margin = y_margin;
        let captured_plot_inset = plot_inset;
        let on_move = cx.listener(move |this, event: &MouseMoveEvent, _window, cx| {
            // Translate window-space pointer x into wrapper-local x using
            // the origin captured during the last paint.
            let local_x = f32::from(event.position.x) - this.wrapper_origin_x.get();
            let next = compute_hover_index(
                local_x,
                captured_width,
                captured_y_margin,
                captured_plot_inset,
                this.max_points(),
            );
            // P0: pixel-level mouse motion fires this listener 60+ times
            // per second. Re-rendering only when the slot actually changes
            // drops every intra-slot move to a no-op.
            if this.pointer_index != next {
                let previous = this.effective_index();
                this.pointer_index = next;
                cx.notify();
                this.fire_selection_if_changed(previous, cx);
            }
        });

        let on_hover = cx.listener(|this, hovered: &bool, _window, cx| {
            if !hovered && this.pointer_index.is_some() {
                let previous = this.effective_index();
                this.pointer_index = None;
                cx.notify();
                this.fire_selection_if_changed(previous, cx);
            }
        });

        // `on_key_down` only fires while the wrapper holds focus (see
        // `track_focus(&focus_handle)` on the wrapper below), so we don't
        // need to gate on `is_focused` explicitly here.
        let on_key = cx.listener(|this, event: &KeyDownEvent, _window, cx| {
            let max = this.max_points();
            if max == 0 {
                return;
            }
            let last = max - 1;
            // `up`/`down` alias `left`/`right` so a vertically-stacked chart
            // layout stays keyboard-reachable without mapping two different
            // axes to the same action.
            let next = match event.keystroke.key.as_str() {
                "left" | "up" => Some(match this.focus_index.or(this.pointer_index) {
                    Some(i) => i.saturating_sub(1),
                    None => 0,
                }),
                "right" | "down" => Some(match this.focus_index.or(this.pointer_index) {
                    Some(i) => (i + 1).min(last),
                    None => 0,
                }),
                "home" => Some(0),
                "end" => Some(last),
                "escape" => None,
                _ => return,
            };
            if this.focus_index != next {
                let previous = this.effective_index();
                this.focus_index = next;
                cx.notify();
                this.fire_selection_if_changed(previous, cx);
            }
            // Consume the keystroke so a parent focus group / workflow pane
            // doesn't also process the arrow/Home/End/Escape.
            cx.stop_propagation();
        });

        let focus_handle = self.focus_handle.clone();
        let is_focused = focus_handle.is_focused(window);

        let mut wrapper = div()
            .id(ElementId::Name(self.id.clone()))
            .track_focus(&focus_handle)
            .w(width)
            .h(height)
            // Match `Chart::render`'s container radius so the focus ring
            // traces the same corners the plot itself is clipped to.
            .rounded(theme.radius_md)
            .relative()
            .child(chart)
            .child(crosshair);

        if let Some(tooltip) = tooltip_el {
            wrapper = wrapper.child(tooltip);
        }

        let on_wheel = cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
            if this.scroll.is_none() {
                return;
            }
            // Visible-window width in data units drives the per-tick step.
            // Falls out to zero when no numeric visible-domain is set, in
            // which case `advance_scroll_position` is a no-op.
            let width = this
                .scroll
                .as_ref()
                .and_then(|c| c.x_visible_domain.as_ref())
                .and_then(|(lo, hi)| Some(hi.as_number()? - lo.as_number()?))
                .unwrap_or(0.0);
            if width <= 0.0 {
                return;
            }
            // Convert the raw wheel delta to data units. The plan calls for
            // ~10% of the visible-window width per line-tick; pixel deltas
            // (macOS trackpads) use a proportional factor that matches the
            // wheel feel at ~40 px / line.
            let delta = match event.delta {
                ScrollDelta::Lines(d) => (d.x + d.y) as f64 * 0.1 * width,
                ScrollDelta::Pixels(d) => (f32::from(d.x) + f32::from(d.y)) as f64 * 0.0025 * width,
            };
            if delta.abs() < f64::EPSILON {
                return;
            }
            if this.advance_scroll_position(delta) {
                cx.notify();
                cx.stop_propagation();
            }
        });

        let wrapper = wrapper
            .on_mouse_move(on_move)
            .on_hover(on_hover)
            .on_key_down(on_key)
            .on_scroll_wheel(on_wheel);

        // Focus ring signals keyboard ownership of the chart so the tabstop
        // is visible before any arrow key lands. Stateless Chart children
        // already have no ring; this handles the outer wrapper.
        apply_focus_ring(wrapper, theme, is_focused, &[])
    }
}

/// Where the crosshair should paint horizontally inside the plot area.
///
/// Line/Area/Range paint data points at `plot_w * i / (n - 1)` (the first
/// at x=0, the last at x=plot_w). Bar/Point/Rule paint at slot centres
/// `slot_w * (i + 0.5)`. Keeping the crosshair over the actual mark — not
/// the slot the pointer lives in — matters most at the plot edges, where a
/// slot-centred crosshair for a Line chart would sit ~half a slot-width
/// away from the first/last data point.
fn crosshair_x_offset(chart_type: ChartType, plot_w: f32, idx: usize, max_pts: usize) -> f32 {
    let is_point_based = matches!(
        chart_type,
        ChartType::Line | ChartType::Area | ChartType::Range
    );
    if is_point_based && max_pts > 1 {
        plot_w * idx as f32 / (max_pts - 1) as f32
    } else {
        let slot_w = plot_w / max_pts.max(1) as f32;
        slot_w * (idx as f32 + 0.5)
    }
}

/// Map a pointer x (relative to the wrapper's left edge) to the hovered
/// data-point slot. Returns `None` when the pointer is inside the plot
/// inset or the Y-label column, past the right inset, or the chart has no
/// data.
fn compute_hover_index(
    local_x: f32,
    width: f32,
    y_margin: f32,
    plot_inset: f32,
    max_points: usize,
) -> Option<usize> {
    if max_points == 0 {
        return None;
    }
    // Plot area sits between `plot_inset + y_margin` on the left and
    // `width - plot_inset` on the right — mirror `Chart::render`.
    let plot_x = local_x - plot_inset - y_margin;
    if plot_x < 0.0 {
        return None;
    }
    let plot_w = (width - 2.0 * plot_inset - y_margin).max(0.0);
    if plot_w <= 0.0 {
        return None;
    }
    let slot_w = plot_w / max_points as f32;
    let idx = (plot_x / slot_w).floor() as usize;
    (idx < max_points).then_some(idx)
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::{ChartType, compute_hover_index, crosshair_x_offset};

    #[test]
    fn hover_with_no_axis_covers_plot_width_minus_insets() {
        // plot area is 200 - 2*8 = 184 wide, 5 slots of 36.8, starting at x=8.
        assert_eq!(compute_hover_index(8.0, 200.0, 0.0, 8.0, 5), Some(0));
        assert_eq!(compute_hover_index(120.0, 200.0, 0.0, 8.0, 5), Some(3));
        assert_eq!(compute_hover_index(191.9, 200.0, 0.0, 8.0, 5), Some(4));
    }

    #[test]
    fn hover_inside_left_inset_returns_none() {
        // plot_inset=8 means the plot starts at x=8; anything left of
        // that is inside the clipped-corner inset.
        assert_eq!(compute_hover_index(0.0, 200.0, 0.0, 8.0, 5), None);
        assert_eq!(compute_hover_index(7.9, 200.0, 0.0, 8.0, 5), None);
        assert_eq!(compute_hover_index(8.0, 200.0, 0.0, 8.0, 5), Some(0));
    }

    #[test]
    fn hover_inside_y_label_column_returns_none() {
        // y_margin=40 + plot_inset=8 means the plot starts at x=48.
        assert_eq!(compute_hover_index(20.0, 256.0, 40.0, 8.0, 5), None);
        assert_eq!(compute_hover_index(47.9, 256.0, 40.0, 8.0, 5), None);
    }

    #[test]
    fn hover_with_axis_offsets_plot_area_left_edge() {
        // plot area is 256 - 40 - 16 = 200 wide, 5 slots of 40, starting at x=48.
        assert_eq!(compute_hover_index(48.0, 256.0, 40.0, 8.0, 5), Some(0));
        assert_eq!(compute_hover_index(168.0, 256.0, 40.0, 8.0, 5), Some(3));
        assert_eq!(compute_hover_index(247.0, 256.0, 40.0, 8.0, 5), Some(4));
    }

    #[test]
    fn hover_with_empty_series_returns_none() {
        assert_eq!(compute_hover_index(100.0, 200.0, 0.0, 8.0, 0), None);
    }

    #[test]
    fn crosshair_bar_uses_slot_center() {
        // 5 slots across 200px → slot_w=40. Slot centers at 20, 60, 100, 140, 180.
        assert_eq!(crosshair_x_offset(ChartType::Bar, 200.0, 0, 5), 20.0);
        assert_eq!(crosshair_x_offset(ChartType::Bar, 200.0, 2, 5), 100.0);
        assert_eq!(crosshair_x_offset(ChartType::Bar, 200.0, 4, 5), 180.0);
    }

    #[test]
    fn crosshair_point_uses_slot_center() {
        // Point marks also draw at slot centers (render_points uses slot_w
        // bands with the dot centered inside each band).
        assert_eq!(crosshair_x_offset(ChartType::Point, 200.0, 0, 5), 20.0);
        assert_eq!(crosshair_x_offset(ChartType::Point, 200.0, 4, 5), 180.0);
    }

    #[test]
    fn crosshair_line_uses_data_points() {
        // Line marks sit at plot_w * i / (n - 1): 0, 50, 100, 150, 200.
        assert_eq!(crosshair_x_offset(ChartType::Line, 200.0, 0, 5), 0.0);
        assert_eq!(crosshair_x_offset(ChartType::Line, 200.0, 2, 5), 100.0);
        assert_eq!(crosshair_x_offset(ChartType::Line, 200.0, 4, 5), 200.0);
    }

    #[test]
    fn crosshair_area_and_range_use_data_points() {
        assert_eq!(crosshair_x_offset(ChartType::Area, 200.0, 4, 5), 200.0);
        assert_eq!(crosshair_x_offset(ChartType::Range, 200.0, 0, 5), 0.0);
    }

    #[test]
    fn crosshair_single_point_line_falls_back_to_slot_center() {
        // With n=1 the data-point formula divides by zero; we fall back
        // to slot-center math which places the mark at plot_w/2.
        assert_eq!(crosshair_x_offset(ChartType::Line, 200.0, 0, 1), 100.0);
    }

    // ─── Selection binding ─────────────────────────────────────────────
    //
    // Phase 9 wires a push-to-parent selection callback into ChartView.
    // The callback fires on any effective-index change (pointer or
    // keyboard focus). Tests below exercise the helper surface used by
    // each listener so the firing contract is verified without having
    // to drive the full event loop.

    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::TestAppContext;

    use crate::components::content::chart::types::{ChartDataSeries, ChartDataSet, ChartSeries};
    use crate::foundations::theme::TahoeTheme;

    use super::{ChartView, SelectedPoint};

    fn setup_cx_with_theme(cx: &mut TestAppContext) {
        cx.update(|cx| {
            if !cx.has_global::<TahoeTheme>() {
                cx.set_global(TahoeTheme::dark());
            }
        });
    }

    fn multi_series_set() -> ChartDataSet {
        ChartDataSet::multi(vec![
            ChartSeries::new(ChartDataSeries::new("A", vec![10.0, 20.0, 30.0])),
            ChartSeries::new(ChartDataSeries::new("B", vec![15.0, 25.0, 35.0])),
        ])
    }

    #[gpui::test]
    async fn selection_binding_field_defaults_to_none(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| ChartView::new(cx, multi_series_set()));
        view.update(cx, |chart, _window, _cx| {
            assert!(chart.selection_binding.is_none());
        })
        .unwrap();
    }

    #[gpui::test]
    async fn selection_binding_builder_stores_callback(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| {
            ChartView::new(cx, multi_series_set()).selection_binding(|_chart, _selection, _cx| {})
        });
        view.update(cx, |chart, _window, _cx| {
            assert!(chart.selection_binding.is_some());
        })
        .unwrap();
    }

    #[gpui::test]
    async fn fire_selection_reports_first_series_point(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let captured: Rc<RefCell<Option<SelectedPoint>>> = Rc::new(RefCell::new(None));
        let captured_cb = captured.clone();
        let view = cx.add_window(|_, cx| {
            ChartView::new(cx, multi_series_set()).selection_binding(
                move |_chart, selection, _cx| {
                    *captured_cb.borrow_mut() = selection;
                },
            )
        });

        view.update(cx, |chart, _window, cx| {
            chart.focus_index = Some(1);
            chart.fire_selection_if_changed(None, cx);
        })
        .unwrap();

        let captured = captured.borrow();
        let selection = captured.as_ref().expect("binding should have fired");
        assert_eq!(selection.series_name.as_ref(), "A");
        assert_eq!(selection.x.as_number_f32(), Some(1.0));
        assert_eq!(selection.y.as_number_f32(), Some(20.0));
    }

    #[gpui::test]
    async fn fire_selection_skips_when_index_unchanged(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let calls = Rc::new(RefCell::new(0u32));
        let calls_cb = calls.clone();
        let view = cx.add_window(|_, cx| {
            ChartView::new(cx, multi_series_set()).selection_binding(
                move |_chart, _selection, _cx| {
                    *calls_cb.borrow_mut() += 1;
                },
            )
        });

        view.update(cx, |chart, _window, cx| {
            chart.focus_index = Some(2);
            // previous == next == Some(2) → binding must NOT fire.
            chart.fire_selection_if_changed(Some(2), cx);
        })
        .unwrap();

        assert_eq!(*calls.borrow(), 0);
    }

    #[gpui::test]
    async fn fire_selection_emits_none_when_cleared(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let captured: Rc<RefCell<Option<Option<SelectedPoint>>>> = Rc::new(RefCell::new(None));
        let captured_cb = captured.clone();
        let view = cx.add_window(|_, cx| {
            ChartView::new(cx, multi_series_set()).selection_binding(
                move |_chart, selection, _cx| {
                    // Wrap in Some(_) so we can distinguish "binding never
                    // fired" from "binding fired with None".
                    *captured_cb.borrow_mut() = Some(selection);
                },
            )
        });

        view.update(cx, |chart, _window, cx| {
            chart.pointer_index = None;
            chart.focus_index = None;
            chart.fire_selection_if_changed(Some(1), cx);
        })
        .unwrap();

        let captured = captured.borrow();
        let outer = captured.as_ref().expect("binding should have fired");
        assert!(outer.is_none(), "binding payload should be None on clear");
    }

    #[gpui::test]
    async fn fire_selection_tolerates_empty_series(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let captured: Rc<RefCell<Option<Option<SelectedPoint>>>> = Rc::new(RefCell::new(None));
        let captured_cb = captured.clone();
        let view = cx.add_window(|_, cx| {
            ChartView::new(
                cx,
                ChartDataSet::multi(vec![ChartSeries::new(ChartDataSeries::new(
                    "Empty",
                    vec![],
                ))]),
            )
            .selection_binding(move |_chart, selection, _cx| {
                *captured_cb.borrow_mut() = Some(selection);
            })
        });

        view.update(cx, |chart, _window, cx| {
            chart.focus_index = Some(0);
            chart.fire_selection_if_changed(None, cx);
        })
        .unwrap();

        let captured = captured.borrow();
        let outer = captured.as_ref().expect("binding should have fired");
        // focus_index was Some(0) but the series has no points, so the
        // payload should be None rather than panicking.
        assert!(outer.is_none());
    }

    // ─── Phase 10: scroll / zoom ───────────────────────────────────────
    //
    // The wheel listener wraps `advance_scroll_position`, which carries the
    // clamp-to-data-extent logic. Testing the helper directly covers the
    // contract without having to synthesise a `ScrollWheelEvent` pipeline.

    use super::ChartScrollConfig;

    fn hundred_point_series() -> ChartDataSet {
        let values: Vec<f32> = (0..100).map(|i| i as f32).collect();
        ChartDataSet::from(ChartDataSeries::new("Long", values))
    }

    #[gpui::test]
    async fn advance_scroll_position_is_noop_without_scroll(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| ChartView::new(cx, hundred_point_series()));
        view.update(cx, |chart, _window, _cx| {
            assert!(!chart.advance_scroll_position(5.0));
        })
        .unwrap();
    }

    #[gpui::test]
    async fn advance_scroll_position_is_noop_without_visible_domain(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        // Scroll config with only a position and no visible-domain — width
        // is zero, so `advance_scroll_position` should short-circuit.
        let view = cx.add_window(|_, cx| {
            ChartView::new(cx, hundred_point_series())
                .scroll(ChartScrollConfig::new().x_scroll_position(0.0))
        });
        view.update(cx, |chart, _window, _cx| {
            assert!(!chart.advance_scroll_position(5.0));
        })
        .unwrap();
    }

    #[gpui::test]
    async fn advance_scroll_position_shifts_forward(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| {
            ChartView::new(cx, hundred_point_series())
                .scroll(ChartScrollConfig::new().x_visible_domain(0.0, 9.0))
        });
        view.update(cx, |chart, _window, _cx| {
            assert!(chart.advance_scroll_position(5.0));
            let pos = chart
                .scroll
                .as_ref()
                .and_then(|c| c.x_scroll_position.as_ref())
                .and_then(|v| v.as_number());
            assert_eq!(pos, Some(5.0));
        })
        .unwrap();
    }

    #[gpui::test]
    async fn advance_scroll_position_clamps_to_upper_bound(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| {
            ChartView::new(cx, hundred_point_series())
                .scroll(ChartScrollConfig::new().x_visible_domain(0.0, 9.0))
        });
        view.update(cx, |chart, _window, _cx| {
            // Request a huge forward jump; must clamp to `d_hi - width`
            // (99 - 9 = 90) rather than sliding past the data.
            assert!(chart.advance_scroll_position(200.0));
            let pos = chart
                .scroll
                .as_ref()
                .and_then(|c| c.x_scroll_position.as_ref())
                .and_then(|v| v.as_number());
            assert_eq!(pos, Some(90.0));
        })
        .unwrap();
    }

    #[gpui::test]
    async fn advance_scroll_position_clamps_to_lower_bound(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| {
            ChartView::new(cx, hundred_point_series()).scroll(
                ChartScrollConfig::new()
                    .x_visible_domain(0.0, 9.0)
                    .x_scroll_position(20.0),
            )
        });
        view.update(cx, |chart, _window, _cx| {
            // Backwards past 0 must clamp to the data low (0.0), not go
            // negative.
            assert!(chart.advance_scroll_position(-100.0));
            let pos = chart
                .scroll
                .as_ref()
                .and_then(|c| c.x_scroll_position.as_ref())
                .and_then(|v| v.as_number());
            assert_eq!(pos, Some(0.0));
        })
        .unwrap();
    }

    // ─── Phase 11: Audio Graphs accessibility ─────────────────────────
    //
    // ChartView threads the descriptor through to the inner Chart on
    // render and exposes `play_audio_graph` so hosts can wire VoiceOver's
    // VO+Shift+S to the sonification path. The tests below verify the
    // builder stores the descriptor, the sonification hook reports its
    // active/inactive state, and calling it without a descriptor is a
    // cheap no-op.

    use super::ChartDescriptor;
    use crate::components::content::chart::audio_graph::AxisDescriptor;

    #[gpui::test]
    async fn audio_graph_field_defaults_to_none(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| ChartView::new(cx, multi_series_set()));
        view.update(cx, |chart, _window, _cx| {
            assert!(chart.audio_graph.is_none());
            assert!(!chart.play_audio_graph());
        })
        .unwrap();
    }

    #[gpui::test]
    async fn audio_graph_builder_stores_descriptor(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| {
            let desc = ChartDescriptor::new(
                "Temperature",
                "Weekly average",
                AxisDescriptor::new("Day", (0.0, 6.0)),
                AxisDescriptor::new("°C", (18.0, 27.0)),
            );
            ChartView::new(cx, multi_series_set()).audio_graph(desc)
        });
        view.update(cx, |chart, _window, _cx| {
            let desc = chart.audio_graph.as_ref().expect("descriptor stored");
            assert_eq!(desc.title.as_ref(), "Temperature");
            assert_eq!(desc.summary.as_ref(), "Weekly average");
            assert!(chart.play_audio_graph());
        })
        .unwrap();
    }

    // ─── Phase 12: Animated data transitions ──────────────────────────
    //
    // `set_data` replaces the live data set and stashes the prior one
    // for a ~300 ms tween. Tests below cover the field-level behaviour:
    // the stash is populated outside reduce-motion, reduce-motion snaps
    // instantly, and `current_render_data_set` blends the two sources
    // while a tween is active.

    use std::time::Instant;

    use super::super::animation::DATA_TRANSITION_DURATION;
    use crate::foundations::accessibility::AccessibilityMode;

    fn setup_cx_with_reduce_motion(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::REDUCE_MOTION;
            cx.set_global(theme);
        });
    }

    #[gpui::test]
    async fn animation_fields_default_to_none(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| ChartView::new(cx, multi_series_set()));
        view.update(cx, |chart, _window, _cx| {
            assert!(chart.previous_data_set.is_none());
            assert!(chart.transition_started_at.is_none());
            assert!(chart.transition_task.is_none());
            assert!(chart.transition_progress().is_none());
        })
        .unwrap();
    }

    #[gpui::test]
    async fn set_data_stashes_previous_and_starts_tween(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| ChartView::new(cx, multi_series_set()));
        view.update(cx, |chart, _window, cx| {
            let next = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("A", vec![100.0, 200.0, 300.0])),
                ChartSeries::new(ChartDataSeries::new("B", vec![150.0, 250.0, 350.0])),
            ]);
            chart.set_data(next, cx);
            assert!(
                chart.previous_data_set.is_some(),
                "prior data should be stashed for tween"
            );
            assert!(
                chart.transition_started_at.is_some(),
                "transition start timestamp should be set"
            );
            assert!(
                chart.transition_task.is_some(),
                "a tick task should be spawned"
            );
            // `data_set` is already the target — `current_render_data_set`
            // is the only spot that blends in the stashed previous.
            assert_eq!(
                chart.data_set.series[0].inner.points[0].y.as_number_f32(),
                Some(100.0)
            );
        })
        .unwrap();
    }

    #[gpui::test]
    async fn set_data_with_reduce_motion_snaps_instantly(cx: &mut TestAppContext) {
        setup_cx_with_reduce_motion(cx);
        let view = cx.add_window(|_, cx| ChartView::new(cx, multi_series_set()));
        view.update(cx, |chart, _window, cx| {
            let next = ChartDataSet::multi(vec![ChartSeries::new(ChartDataSeries::new(
                "Snap",
                vec![100.0, 200.0, 300.0],
            ))]);
            chart.set_data(next, cx);
            assert!(
                chart.previous_data_set.is_none(),
                "reduce-motion must not stash previous"
            );
            assert!(chart.transition_started_at.is_none());
            assert!(chart.transition_task.is_none());
            assert_eq!(chart.data_set.series[0].inner.name.as_ref(), "Snap");
        })
        .unwrap();
    }

    #[gpui::test]
    async fn current_render_data_set_blends_while_tween_active(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| ChartView::new(cx, multi_series_set()));
        view.update(cx, |chart, _window, _cx| {
            // Construct the tween state by hand so we can freeze progress
            // at 0.5 without waiting on the wall clock. `current_render_data_set`
            // is what the render path actually reads.
            let prev = ChartDataSet::multi(vec![ChartSeries::new(ChartDataSeries::new(
                "S",
                vec![0.0, 0.0],
            ))]);
            let next = ChartDataSet::multi(vec![ChartSeries::new(ChartDataSeries::new(
                "S",
                vec![100.0, 100.0],
            ))]);
            chart.data_set = next;
            chart.previous_data_set = Some(prev);
            chart.transition_started_at = Some(Instant::now() - DATA_TRANSITION_DURATION / 2);

            let blended = chart.current_render_data_set();
            // Midpoint → y lerps to ~50.
            let y = blended.series[0].inner.points[0].y.as_number_f32().unwrap();
            assert!(
                (y - 50.0).abs() < 5.0,
                "midpoint lerp expected ~50, got {y}"
            );
        })
        .unwrap();
    }

    #[gpui::test]
    async fn current_render_data_set_returns_target_after_duration(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| ChartView::new(cx, multi_series_set()));
        view.update(cx, |chart, _window, _cx| {
            let prev =
                ChartDataSet::multi(vec![ChartSeries::new(ChartDataSeries::new("S", vec![0.0]))]);
            let next = ChartDataSet::multi(vec![ChartSeries::new(ChartDataSeries::new(
                "S",
                vec![99.0],
            ))]);
            chart.data_set = next;
            chart.previous_data_set = Some(prev);
            // Start in the distant past so progress clamps to >= 1.0.
            chart.transition_started_at = Some(Instant::now() - DATA_TRANSITION_DURATION * 3);

            let settled = chart.current_render_data_set();
            assert_eq!(
                settled.series[0].inner.points[0].y.as_number_f32(),
                Some(99.0)
            );
        })
        .unwrap();
    }

    #[gpui::test]
    async fn transition_progress_clamps_to_one_after_duration(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| ChartView::new(cx, multi_series_set()));
        view.update(cx, |chart, _window, _cx| {
            chart.transition_started_at = Some(Instant::now() - DATA_TRANSITION_DURATION * 2);
            let p = chart.transition_progress().expect("transition active");
            assert!((p - 1.0).abs() < f32::EPSILON);
        })
        .unwrap();
    }

    #[gpui::test]
    async fn set_data_mid_flight_re_stashes_interpolated_snapshot(cx: &mut TestAppContext) {
        setup_cx_with_theme(cx);
        let view = cx.add_window(|_, cx| {
            ChartView::new(
                cx,
                ChartDataSet::multi(vec![ChartSeries::new(ChartDataSeries::new("S", vec![0.0]))]),
            )
        });
        view.update(cx, |chart, _window, cx| {
            // First tween: 0 → 100, frozen midway.
            chart.set_data(
                ChartDataSet::multi(vec![ChartSeries::new(ChartDataSeries::new(
                    "S",
                    vec![100.0],
                ))]),
                cx,
            );
            chart.transition_started_at = Some(Instant::now() - DATA_TRANSITION_DURATION / 2);

            // Second tween at mid-flight: the stashed previous should be
            // the interpolated snapshot (~50), not the original 0.
            chart.set_data(
                ChartDataSet::multi(vec![ChartSeries::new(ChartDataSeries::new(
                    "S",
                    vec![200.0],
                ))]),
                cx,
            );
            let stashed = chart
                .previous_data_set
                .as_ref()
                .expect("second tween should stash previous");
            let stashed_y = stashed.series[0].inner.points[0].y.as_number_f32().unwrap();
            assert!(
                (40.0..=60.0).contains(&stashed_y),
                "expected ~50 from mid-flight capture, got {stashed_y}"
            );
        })
        .unwrap();
    }
}
