//! Chart demo for the primitive gallery.
//!
//! <https://developer.apple.com/design/human-interface-guidelines/charts>

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::content::chart::{Chart, ChartDataSeries, ChartType};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let sales = ChartDataSeries::new("Sales", vec![10.0, 20.0, 15.0, 30.0, 25.0]);
    let trend = ChartDataSeries::new("Trend", vec![5.0, 12.0, 8.0, 22.0, 18.0, 30.0, 25.0]);
    let empty = ChartDataSeries::new("Empty", vec![]);

    div()
        .id("charts-pane")
        .child(
            div()
                .p(theme.spacing_xl)
                .flex()
                .flex_col()
                .gap(theme.spacing_lg)
                .child(
                    div()
                        .text_style_emphasized(TextStyle::LargeTitle, theme)
                        .text_color(theme.text)
                        .child("Charts"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Minimal Bar and Line chart primitives aligned with \
                             the HIG Charts page. v1 scope: single-series, \
                             no axes or gridlines.",
                        ),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Bar chart"),
                )
                .child(
                    Chart::new(sales.clone())
                        .id("bar-chart")
                        .chart_type(ChartType::Bar),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Line chart (point sparkline)"),
                )
                .child(
                    Chart::new(trend.clone())
                        .id("line-chart")
                        .chart_type(ChartType::Line)
                        .size(px(320.0), px(120.0)),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Point chart"),
                )
                .child(
                    Chart::new(sales.clone())
                        .id("point-chart")
                        .chart_type(ChartType::Point)
                        .size(px(200.0), px(80.0)),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Area chart"),
                )
                .child(
                    Chart::new(sales.clone())
                        .id("area-chart")
                        .chart_type(ChartType::Area),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Range chart"),
                )
                .child(
                    Chart::new(ChartDataSeries::range(
                        "Confidence",
                        vec![5.0, 12.0, 8.0, 22.0, 18.0],
                        vec![15.0, 28.0, 22.0, 38.0, 32.0],
                    ))
                    .id("range-chart")
                    .chart_type(ChartType::Range),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Rule chart"),
                )
                .child(
                    Chart::new(trend)
                        .id("rule-chart")
                        .chart_type(ChartType::Rule),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Empty series"),
                )
                .child(
                    Chart::new(empty)
                        .id("empty-chart")
                        .chart_type(ChartType::Bar),
                ),
        )
        .into_any_element()
}
