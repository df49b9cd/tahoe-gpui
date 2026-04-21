//! Stateful chart with interactive hover tooltip.

use gpui::prelude::*;
use gpui::{
    Context, ElementId, FocusHandle, Hsla, IntoElement, MouseMoveEvent, Pixels, SharedString,
    Window, canvas, div, px,
};

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

        let crosshair_color = theme.text_muted;
        let crosshair = canvas(
            |_info, _window, _cx| {},
            move |bounds, _state, window, _cx| {
                let Some(idx) = hover_index else { return };
                if max_pts == 0 {
                    return;
                }
                let w = f32::from(bounds.size.width);
                let slot_w = w / max_pts as f32;
                let x = bounds.origin.x + gpui::px(slot_w * (idx as f32 + 0.5));

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
                .flex_col();

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

        let max_pts_capture = self.max_points();
        let on_move = cx.listener(move |this, event: &MouseMoveEvent, _window, cx| {
            if max_pts_capture == 0 {
                this.hover_index = None;
                cx.notify();
                return;
            }
            let local_x = f32::from(event.position.x);
            let slot_w = f32::from(this.width) / max_pts_capture as f32;
            let idx = (local_x / slot_w).floor() as usize;
            this.hover_index = if idx < max_pts_capture {
                Some(idx)
            } else {
                None
            };
            cx.notify();
        });

        let on_hover = cx.listener(|this, hovered: &bool, _window, cx| {
            if !hovered {
                this.hover_index = None;
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

        wrapper.on_mouse_move(on_move).on_hover(on_hover)
    }
}
