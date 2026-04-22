//! Stateful chart with interactive hover tooltip.

use std::cell::Cell;
use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    Context, ElementId, FocusHandle, Hsla, IntoElement, KeyDownEvent, MouseMoveEvent, Pixels,
    SharedString, Window, canvas, div, px,
};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::ActiveTheme;
use crate::foundations::typography::{TextStyle, TextStyledExt};

use super::render::series_color;
use super::types::{AxisConfig, ChartDataSet, ChartType, GridlineConfig};

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
}

impl ChartView {
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

    fn max_points(&self) -> usize {
        self.data_set
            .series
            .iter()
            .map(|s| s.inner.values.len())
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
}

impl Render for ChartView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        let mut chart = super::Chart::new(self.data_set.clone())
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

        // Pointer wins when both are live (the user is actively moving the
        // mouse), otherwise fall back to the last keyboard-selected slot.
        let hover_index = self.pointer_index.or(self.focus_index);
        let data_set = self.data_set.clone();
        let global_color = self.global_color;
        let width = self.width;
        let height = self.height;
        let max_pts = self.max_points();
        let y_margin = self.y_margin(theme);
        let plot_inset = self.plot_inset(theme);

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
                // column).
                let plot_w = (f32::from(bounds.size.width) - 2.0 * plot_inset - y_margin).max(0.0);
                let slot_w = plot_w / max_pts as f32;
                let x =
                    bounds.origin.x + gpui::px(plot_inset + y_margin + slot_w * (idx as f32 + 0.5));

                // Keep the crosshair vertically inside the plot area so it
                // matches where data paints (Chart insets by `plot_inset`
                // on top and bottom too).
                let top = bounds.origin.y + gpui::px(plot_inset);
                let bottom = bounds.origin.y
                    + gpui::px((f32::from(bounds.size.height) - plot_inset).max(0.0));
                let mut pb = gpui::PathBuilder::stroke(gpui::px(1.0));
                pb.move_to(gpui::point(x, top));
                pb.line_to(gpui::point(x, bottom));
                if let Ok(path) = pb.build() {
                    window.paint_path(path, crosshair_color);
                }
            },
        )
        .w(width)
        .h(height);

        let tooltip_el = if let Some(idx) = hover_index {
            let mut items = Vec::new();
            for (si, series) in data_set.series.iter().enumerate() {
                let value = series.inner.values.get(idx).copied();
                let color = series_color(&data_set, global_color, si, theme);
                items.push((series.inner.name.clone(), value, color));
            }

            // Build a single concatenated VoiceOver label so the tooltip
            // carries a Tooltip role with meaningful content. Without this
            // the hover value is invisible to assistive tech.
            let tooltip_label: SharedString = SharedString::from(
                items
                    .iter()
                    .map(|(name, value, _)| match value {
                        Some(v) => format!("{name}: {v:.1}"),
                        None => format!("{name}: —"),
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            let a11y = AccessibilityProps::new()
                .role(AccessibilityRole::Tooltip)
                .label(tooltip_label);

            let mut tooltip_div = div()
                .absolute()
                .top(px(4.0))
                .right(px(4.0))
                .bg(theme.surface)
                .rounded(theme.radius_sm)
                .border_1()
                .border_color(theme.border)
                .p(px(6.0))
                .gap(px(2.0))
                .flex()
                .flex_col()
                .with_accessibility(&a11y);

            for (name, value, color) in items {
                let label = match value {
                    Some(v) => format!("{name}: {v:.1}"),
                    None => format!("{name}: —"),
                };
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
                this.pointer_index = next;
                cx.notify();
            }
        });

        let on_hover = cx.listener(|this, hovered: &bool, _window, cx| {
            if !hovered && this.pointer_index.is_some() {
                this.pointer_index = None;
                cx.notify();
            }
        });

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
                this.focus_index = next;
                cx.notify();
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
            .relative()
            .child(chart)
            .child(crosshair);

        if let Some(tooltip) = tooltip_el {
            wrapper = wrapper.child(tooltip);
        }

        let wrapper = wrapper
            .on_mouse_move(on_move)
            .on_hover(on_hover)
            .on_key_down(on_key);

        // Focus ring signals keyboard ownership of the chart so the tabstop
        // is visible before any arrow key lands. Stateless Chart children
        // already have no ring; this handles the outer wrapper.
        apply_focus_ring(wrapper, theme, is_focused, &[])
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

    use super::compute_hover_index;

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
}
