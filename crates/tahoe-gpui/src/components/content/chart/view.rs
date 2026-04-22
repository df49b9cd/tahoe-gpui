//! Stateful chart with interactive hover tooltip.

use gpui::prelude::*;
use gpui::{
    Context, ElementId, FocusHandle, Hsla, IntoElement, KeyDownEvent, MouseMoveEvent, Pixels,
    SharedString, Window, canvas, div, px,
};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
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
    hover_index: Option<usize>,
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
            hover_index: None,
        }
    }

    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
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
        self.global_color = Some(color);
        self
    }

    pub fn axis(mut self, config: AxisConfig) -> Self {
        self.axis = Some(config);
        self
    }

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
    /// correct data-point slot when an axis is configured.
    fn y_margin(&self) -> f32 {
        if self
            .axis
            .as_ref()
            .is_some_and(|a| a.y_tick_count > 0 || a.y_ticks.is_some())
        {
            AxisConfig::Y_LABEL_WIDTH
        } else {
            0.0
        }
    }
}

impl Render for ChartView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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

        let hover_index = self.hover_index;
        let data_set = self.data_set.clone();
        let global_color = self.global_color;
        let width = self.width;
        let height = self.height;
        let max_pts = self.max_points();
        let y_margin = self.y_margin();

        let crosshair_color = theme.text_muted;
        let crosshair = canvas(
            |_info, _window, _cx| {},
            move |bounds, _state, window, _cx| {
                let Some(idx) = hover_index else { return };
                if max_pts == 0 {
                    return;
                }
                // Crosshair x lives inside the plot area (wrapper width
                // minus the Y-label column).
                let plot_w = (f32::from(bounds.size.width) - y_margin).max(0.0);
                let slot_w = plot_w / max_pts as f32;
                let x = bounds.origin.x + gpui::px(y_margin + slot_w * (idx as f32 + 0.5));

                let mut pb = gpui::PathBuilder::stroke(gpui::px(1.0));
                pb.move_to(gpui::point(x, bounds.origin.y));
                pb.line_to(gpui::point(
                    x,
                    bounds.origin.y + gpui::px(f32::from(bounds.size.height)),
                ));
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

        let on_move = cx.listener(move |this, event: &MouseMoveEvent, _window, cx| {
            let next = compute_hover_index(
                f32::from(event.position.x),
                f32::from(this.width),
                this.y_margin(),
                this.max_points(),
            );
            // P0: pixel-level mouse motion fires this listener 60+ times
            // per second. Re-rendering only when the slot actually changes
            // drops every intra-slot move to a no-op.
            if this.hover_index != next {
                this.hover_index = next;
                cx.notify();
            }
        });

        let on_hover = cx.listener(|this, hovered: &bool, _window, cx| {
            if !hovered && this.hover_index.is_some() {
                this.hover_index = None;
                cx.notify();
            }
        });

        let on_key = cx.listener(|this, event: &KeyDownEvent, _window, cx| {
            let max = this.max_points();
            if max == 0 {
                return;
            }
            let last = max - 1;
            let next = match event.keystroke.key.as_str() {
                "left" => Some(match this.hover_index {
                    Some(i) => i.saturating_sub(1),
                    None => 0,
                }),
                "right" => Some(match this.hover_index {
                    Some(i) => (i + 1).min(last),
                    None => 0,
                }),
                "home" => Some(0),
                "end" => Some(last),
                "escape" => None,
                _ => return,
            };
            if this.hover_index != next {
                this.hover_index = next;
                cx.notify();
            }
        });

        let focus_handle = self.focus_handle.clone();

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

        wrapper
            .on_mouse_move(on_move)
            .on_hover(on_hover)
            .on_key_down(on_key)
    }
}

/// Map a pointer x (relative to the wrapper's left edge) to the hovered
/// data-point slot. Returns `None` when the pointer is inside the Y-label
/// column, past the right edge, or the chart has no data.
fn compute_hover_index(
    local_x: f32,
    width: f32,
    y_margin: f32,
    max_points: usize,
) -> Option<usize> {
    if max_points == 0 {
        return None;
    }
    let plot_x = local_x - y_margin;
    if plot_x < 0.0 {
        return None;
    }
    let plot_w = (width - y_margin).max(0.0);
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
    fn hover_with_no_axis_covers_full_width() {
        assert_eq!(compute_hover_index(0.0, 200.0, 0.0, 5), Some(0));
        assert_eq!(compute_hover_index(120.0, 200.0, 0.0, 5), Some(3));
        assert_eq!(compute_hover_index(199.9, 200.0, 0.0, 5), Some(4));
    }

    #[test]
    fn hover_inside_y_label_column_returns_none() {
        // y_margin = 40 means the plot starts at x=40.
        assert_eq!(compute_hover_index(20.0, 240.0, 40.0, 5), None);
        assert_eq!(compute_hover_index(39.9, 240.0, 40.0, 5), None);
    }

    #[test]
    fn hover_with_axis_offsets_plot_area_left_edge() {
        // plot area is 240 - 40 = 200 wide, 5 slots of 40.
        assert_eq!(compute_hover_index(40.0, 240.0, 40.0, 5), Some(0));
        assert_eq!(compute_hover_index(160.0, 240.0, 40.0, 5), Some(3));
        assert_eq!(compute_hover_index(239.0, 240.0, 40.0, 5), Some(4));
    }

    #[test]
    fn hover_with_empty_series_returns_none() {
        assert_eq!(compute_hover_index(100.0, 200.0, 0.0, 0), None);
    }
}
