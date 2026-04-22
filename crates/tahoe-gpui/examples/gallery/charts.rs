//! Chart demo for the primitive gallery.
//!
//! <https://developer.apple.com/design/human-interface-guidelines/charts>

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::content::chart::{
    AxisConfig, Chart, ChartDataSeries, ChartDataSet, ChartSeries, ChartType, GridlineConfig,
};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let sales = ChartDataSeries::new("Sales", vec![10.0, 20.0, 15.0, 30.0, 25.0]);
    let trend = ChartDataSeries::new("Trend", vec![5.0, 12.0, 8.0, 22.0, 18.0, 30.0, 25.0]);
    let target = ChartDataSeries::new("Target", vec![50.0]);
    let empty = ChartDataSeries::new("Empty", vec![]);

    // Multi-series data used by the legend + axis + gridlines demo.
    let multi = ChartDataSet::multi(vec![
        ChartSeries::new(ChartDataSeries::new(
            "Sales",
            vec![10.0, 20.0, 15.0, 30.0, 25.0],
        )),
        ChartSeries::new(ChartDataSeries::new(
            "Target",
            vec![12.0, 18.0, 20.0, 25.0, 22.0],
        )),
    ]);

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
                            "Bar, Line, Area, Point, Range, and Rule marks \
                             aligned with the HIG Charts page. Axes, \
                             gridlines, legends, titles, and an interactive \
                             ChartView with hover crosshair are included below.",
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
                    Chart::new(target)
                        .id("rule-chart")
                        .chart_type(ChartType::Rule),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Multi-series with axis + gridlines + legend"),
                )
                .child(
                    Chart::new(multi.clone())
                        .id("multi-chart")
                        .chart_type(ChartType::Bar)
                        .size(px(360.0), px(200.0))
                        .axis(
                            AxisConfig::new()
                                .y_tick_count(5)
                                .x_labels(vec!["Mon", "Tue", "Wed", "Thu", "Fri"])
                                .show_y_line()
                                .x_baseline(),
                        )
                        .gridlines(GridlineConfig::horizontal()),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Chart with title and subtitle"),
                )
                .child(
                    Chart::new(trend.clone())
                        .id("titled-chart")
                        .chart_type(ChartType::Area)
                        .size(px(360.0), px(200.0))
                        .title("Weekly sales trend")
                        .subtitle("Last 7 days, in thousands"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Interactive ChartView (hover / arrows / Home / End)"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Move the pointer across the chart to see the \
                             crosshair and value tooltip. Tab in and use \
                             arrow keys, Home, and End to navigate by \
                             keyboard; Escape clears the selection.",
                        ),
                )
                .child(state.chart_view.clone())
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
