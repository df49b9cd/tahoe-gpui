//! Chart demo for the primitive gallery.
//!
//! <https://developer.apple.com/design/human-interface-guidelines/charts>

use std::time::{Duration, UNIX_EPOCH};

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::content::chart::{
    AnnotationPosition, AnnotationTarget, AxisConfig, AxisDescriptor, AxisPosition, AxisTickStyle,
    BarOrientation, Chart, ChartAnnotation, ChartDataSeries, ChartDataSet, ChartDescriptor,
    ChartPoint, ChartScrollConfig, ChartSeries, ChartType, DateScale, GridLineStyle,
    GridlineConfig, InterpolationMethod, LegendPosition, LinearScale, LogScale, MarkStackingMethod,
    SeriesDescriptor,
};
use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
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

    // Log-scale demo — ticks land on powers of 10 evenly across 1..1e6.
    let log_growth = ChartDataSeries::new(
        "Throughput",
        vec![
            2.0, 12.0, 85.0, 540.0, 4_200.0, 31_000.0, 240_000.0, 800_000.0,
        ],
    );

    // Date-scale demos — one point per day over a week, then one per month for a year.
    let day = Duration::from_secs(24 * 60 * 60);
    let week_start = UNIX_EPOCH + Duration::from_secs(1_735_689_600); // 2025-01-01
    let week_points: Vec<ChartPoint> = [18.0, 22.0, 19.0, 24.0, 27.0, 26.0, 23.0]
        .iter()
        .enumerate()
        .map(|(i, v)| ChartPoint::new(week_start + day * i as u32, *v))
        .collect();
    let week_series = ChartDataSeries::from_points("Temperature", week_points);
    let week_end = week_start + day * 6;

    let year_start = UNIX_EPOCH + Duration::from_secs(1_704_067_200); // 2024-01-01
    let year_points: Vec<ChartPoint> = [
        42.0, 48.0, 63.0, 74.0, 85.0, 92.0, 95.0, 91.0, 82.0, 69.0, 54.0, 46.0,
    ]
    .iter()
    .enumerate()
    .map(|(i, v)| {
        // Roughly one point every ~30 days across the year.
        let offset_days = (i as u32) * 30;
        ChartPoint::new(year_start + day * offset_days, *v)
    })
    .collect();
    let year_series = ChartDataSeries::from_points("Monthly average", year_points);
    let year_end = year_start + day * 30 * 11;

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
                        .child("Stacked bar chart (Standard)"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Three series accumulate from the baseline — the \
                             total height of each column is the per-slot sum.",
                        ),
                )
                .child(
                    Chart::new(ChartDataSet::multi(vec![
                        ChartSeries::new(ChartDataSeries::new(
                            "Backlog",
                            vec![12.0, 18.0, 10.0, 22.0, 20.0],
                        )),
                        ChartSeries::new(ChartDataSeries::new(
                            "Open",
                            vec![20.0, 22.0, 25.0, 18.0, 24.0],
                        )),
                        ChartSeries::new(ChartDataSeries::new(
                            "Closed",
                            vec![8.0, 14.0, 16.0, 20.0, 28.0],
                        )),
                    ]))
                    .id("stacked-bar-chart")
                    .chart_type(ChartType::Bar)
                    .stacking(MarkStackingMethod::Standard)
                    .size(px(360.0), px(200.0))
                    .axis(
                        AxisConfig::new()
                            .y_tick_count(4)
                            .x_labels(vec!["Mon", "Tue", "Wed", "Thu", "Fri"]),
                    )
                    .gridlines(GridlineConfig::horizontal()),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Normalised stacked area (100%)"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Each column normalises to 100% — shows the \
                             composition across slots rather than absolute \
                             magnitude.",
                        ),
                )
                .child(
                    Chart::new(ChartDataSet::multi(vec![
                        ChartSeries::new(ChartDataSeries::new(
                            "iOS",
                            vec![35.0, 42.0, 50.0, 55.0, 58.0],
                        )),
                        ChartSeries::new(ChartDataSeries::new(
                            "Android",
                            vec![40.0, 38.0, 33.0, 30.0, 28.0],
                        )),
                        ChartSeries::new(ChartDataSeries::new(
                            "Web",
                            vec![25.0, 20.0, 17.0, 15.0, 14.0],
                        )),
                    ]))
                    .id("normalized-area-chart")
                    .chart_type(ChartType::Area)
                    .stacking(MarkStackingMethod::Normalized)
                    .size(px(360.0), px(200.0)),
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
                        .child("Animated data transitions"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Click the button to swap the ChartView's dataset. \
                             ChartView::set_data springs each point between its \
                             old and new value over ~300 ms (HIG medium ramp). \
                             When Reduce Motion is enabled, the chart snaps to \
                             the new state instead of tweening.",
                        ),
                )
                .child(
                    Button::new("chart-swap-data")
                        .label(if state.chart_tween_alternate {
                            "Show original"
                        } else {
                            "Show what-if"
                        })
                        .variant(ButtonVariant::Primary)
                        .size(ButtonSize::Regular)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.chart_tween_alternate = !this.chart_tween_alternate;
                            let next = if this.chart_tween_alternate {
                                ChartDataSet::multi(vec![
                                    ChartSeries::new(ChartDataSeries::new(
                                        "Sales",
                                        vec![5.0, 35.0, 10.0, 40.0, 15.0, 45.0, 18.0],
                                    )),
                                    ChartSeries::new(ChartDataSeries::new(
                                        "Target",
                                        vec![22.0, 28.0, 30.0, 35.0, 36.0, 34.0, 38.0],
                                    )),
                                ])
                            } else {
                                ChartDataSet::multi(vec![
                                    ChartSeries::new(ChartDataSeries::new(
                                        "Sales",
                                        vec![10.0, 20.0, 15.0, 30.0, 25.0, 28.0, 22.0],
                                    )),
                                    ChartSeries::new(ChartDataSeries::new(
                                        "Target",
                                        vec![12.0, 18.0, 20.0, 25.0, 26.0, 24.0, 28.0],
                                    )),
                                ])
                            };
                            this.chart_view.update(cx, |chart, cx| {
                                chart.set_data(next, cx);
                            });
                            cx.notify();
                        })),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Log Y scale (1 → 1,000,000)"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Y ticks are evenly spaced on the log axis, landing \
                             on powers of 10.",
                        ),
                )
                .child(
                    Chart::new(log_growth)
                        .id("log-scale-chart")
                        .chart_type(ChartType::Line)
                        .size(px(360.0), px(200.0))
                        .axis(
                            AxisConfig::new()
                                .y_scale(LogScale::new(1.0, 1_000_000.0))
                                .show_y_line(),
                        )
                        .gridlines(GridlineConfig::horizontal()),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Date X scale — one week"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "A seven-day domain produces day-level ticks. The \
                             scale is locale-aware and rounds to calendar \
                             boundaries.",
                        ),
                )
                .child(
                    Chart::new(week_series)
                        .id("date-week-chart")
                        .chart_type(ChartType::Line)
                        .size(px(360.0), px(180.0))
                        .axis(
                            AxisConfig::new()
                                .x_scale(DateScale::new(week_start, week_end))
                                .show_y_line(),
                        )
                        .gridlines(GridlineConfig::horizontal()),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Date X scale — one year"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "A year-long domain coarsens automatically — ticks \
                             land on month (or quarter) boundaries.",
                        ),
                )
                .child(
                    Chart::new(year_series)
                        .id("date-year-chart")
                        .chart_type(ChartType::Area)
                        .size(px(360.0), px(180.0))
                        .axis(
                            AxisConfig::new()
                                .x_scale(DateScale::new(year_start, year_end))
                                .show_y_line(),
                        )
                        .gridlines(GridlineConfig::horizontal()),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Pie chart (sectors)"),
                )
                .child(
                    Chart::new(ChartDataSet::multi(vec![
                        ChartSeries::new(ChartDataSeries::new("Reds", vec![28.0])),
                        ChartSeries::new(ChartDataSeries::new("Blues", vec![42.0])),
                        ChartSeries::new(ChartDataSeries::new("Greens", vec![18.0])),
                        ChartSeries::new(ChartDataSeries::new("Other", vec![12.0])),
                    ]))
                    .id("pie-chart")
                    .chart_type(ChartType::Sector)
                    .size(px(220.0), px(220.0)),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Donut chart"),
                )
                .child(
                    Chart::new(ChartDataSet::multi(vec![
                        ChartSeries::new(ChartDataSeries::new("Done", vec![68.0])),
                        ChartSeries::new(ChartDataSeries::new("In progress", vec![22.0])),
                        ChartSeries::new(ChartDataSeries::new("Blocked", vec![10.0])),
                    ]))
                    .id("donut-chart")
                    .chart_type(ChartType::Sector)
                    .inner_radius_ratio(0.6)
                    .size(px(220.0), px(220.0)),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Heatmap (rectangle marks)"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "5×5 cell grid. Each point's z channel drives the \
                             cell's lightness against the base color.",
                        ),
                )
                .child({
                    let mut points = Vec::new();
                    for x in 0..5 {
                        for y in 0..5 {
                            let z = ((x as f32) - 2.0).powi(2) + ((y as f32) - 2.0).powi(2);
                            points.push(ChartPoint::new(x as f32, y as f32).with_z(z));
                        }
                    }
                    Chart::new(ChartDataSeries::from_points("Activity", points))
                        .id("heat-chart")
                        .chart_type(ChartType::Rectangle)
                        .size(px(240.0), px(240.0))
                })
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Horizontal bars"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Swaps the X/Y axes — slots run top-to-bottom, \
                             bars grow from the leading edge.",
                        ),
                )
                .child(
                    Chart::new(sales.clone())
                        .id("horizontal-bar-chart")
                        .chart_type(ChartType::Bar)
                        .bar_orientation(BarOrientation::Horizontal)
                        .size(px(360.0), px(200.0)),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Stacked horizontal bars"),
                )
                .child(
                    Chart::new(ChartDataSet::multi(vec![
                        ChartSeries::new(ChartDataSeries::new(
                            "Backlog",
                            vec![12.0, 18.0, 10.0, 22.0, 20.0],
                        )),
                        ChartSeries::new(ChartDataSeries::new(
                            "Open",
                            vec![20.0, 22.0, 25.0, 18.0, 24.0],
                        )),
                        ChartSeries::new(ChartDataSeries::new(
                            "Closed",
                            vec![8.0, 14.0, 16.0, 20.0, 28.0],
                        )),
                    ]))
                    .id("stacked-horizontal-chart")
                    .chart_type(ChartType::Bar)
                    .bar_orientation(BarOrientation::Horizontal)
                    .stacking(MarkStackingMethod::Standard)
                    .size(px(360.0), px(200.0)),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("2D scatter"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Point charts with an explicit x-scale plot at \
                             real (x, y) coordinates rather than along slot \
                             indices.",
                        ),
                )
                .child({
                    let points: Vec<ChartPoint> = [
                        (1.2, 4.1),
                        (2.8, 6.7),
                        (3.4, 2.3),
                        (4.9, 8.5),
                        (5.5, 5.8),
                        (6.3, 3.2),
                        (7.1, 7.4),
                        (8.6, 9.0),
                        (9.3, 6.1),
                        (1.7, 8.2),
                        (5.1, 1.9),
                        (7.8, 4.6),
                    ]
                    .into_iter()
                    .map(|(x, y)| ChartPoint::new(x, y))
                    .collect();
                    Chart::new(ChartDataSeries::from_points("Observations", points))
                        .id("scatter-chart")
                        .chart_type(ChartType::Point)
                        .size(px(360.0), px(220.0))
                        .axis(
                            AxisConfig::new()
                                .y_tick_count(5)
                                .x_scale(LinearScale::new(0.0, 10.0))
                                .y_scale(LinearScale::new(0.0, 10.0))
                                .show_y_line()
                                .x_baseline(),
                        )
                        .gridlines(GridlineConfig::horizontal())
                })
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Annotations"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Pin descriptive text or an icon to a specific mark — \
                             Swift Charts' .annotation() surface. Overflow-aware \
                             placement flips positions away from the plot edges.",
                        ),
                )
                .child(
                    Chart::new(ChartDataSeries::new(
                        "Revenue",
                        vec![12.0, 18.0, 10.0, 22.0, 36.0, 28.0, 24.0],
                    ))
                    .id("annotated-chart")
                    .chart_type(ChartType::Line)
                    .size(px(360.0), px(180.0))
                    .gridlines(GridlineConfig::horizontal())
                    .axis(AxisConfig::new().y_tick_count(5))
                    .annotations(vec![ChartAnnotation::text(
                        AnnotationTarget::DataPoint {
                            series_idx: 0,
                            point_idx: 4,
                        },
                        AnnotationPosition::Top,
                        "Record high",
                    )]),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Interpolation methods"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Switch how adjacent data points are joined — Swift Charts' \
                             .interpolationMethod() modifier. Monotone keeps the curve \
                             from dipping below the local min; step variants draw \
                             discrete-state time series.",
                        ),
                )
                .child({
                    let interp_series =
                        ChartDataSeries::new("Signal", vec![4.0, 6.0, 3.0, 9.0, 7.0, 11.0, 8.0]);
                    div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .gap(px(16.0))
                        .child(
                            Chart::new(interp_series.clone())
                                .id("interp-catmull")
                                .chart_type(ChartType::Line)
                                .size(px(220.0), px(120.0))
                                .interpolation(InterpolationMethod::CatmullRom)
                                .title("Catmull-Rom"),
                        )
                        .child(
                            Chart::new(interp_series.clone())
                                .id("interp-linear")
                                .chart_type(ChartType::Line)
                                .size(px(220.0), px(120.0))
                                .interpolation(InterpolationMethod::Linear)
                                .title("Linear"),
                        )
                        .child(
                            Chart::new(interp_series.clone())
                                .id("interp-monotone")
                                .chart_type(ChartType::Line)
                                .size(px(220.0), px(120.0))
                                .interpolation(InterpolationMethod::Monotone)
                                .title("Monotone"),
                        )
                        .child(
                            Chart::new(interp_series.clone())
                                .id("interp-step-end")
                                .chart_type(ChartType::Line)
                                .size(px(220.0), px(120.0))
                                .interpolation(InterpolationMethod::StepEnd)
                                .title("Step end"),
                        )
                        .child(
                            Chart::new(interp_series.clone())
                                .id("interp-step-center")
                                .chart_type(ChartType::Line)
                                .size(px(220.0), px(120.0))
                                .interpolation(InterpolationMethod::StepCenter)
                                .title("Step center"),
                        )
                        .child(
                            Chart::new(interp_series)
                                .id("interp-cardinal-loose")
                                .chart_type(ChartType::Line)
                                .size(px(220.0), px(120.0))
                                .interpolation(InterpolationMethod::Cardinal(0.5))
                                .title("Cardinal(0.5)"),
                        )
                })
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Axis customisation"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Mirror Swift Charts' AxisMarks surface — trailing \
                             Y-axis, dashed gridlines, custom value formatters, \
                             and hidden tick marks.",
                        ),
                )
                .child(
                    Chart::new(sales.clone())
                        .id("trailing-y-chart")
                        .chart_type(ChartType::Bar)
                        .size(px(320.0), px(160.0))
                        .axis(
                            AxisConfig::new()
                                .y_tick_count(4)
                                .y_position(AxisPosition::Trailing)
                                .x_labels(vec!["Mon", "Tue", "Wed", "Thu", "Fri"]),
                        )
                        .gridlines(GridlineConfig::horizontal())
                        .title("Trailing Y-axis"),
                )
                .child(
                    Chart::new(trend.clone())
                        .id("dashed-grid-chart")
                        .chart_type(ChartType::Line)
                        .size(px(320.0), px(160.0))
                        .axis(
                            AxisConfig::new()
                                .y_tick_count(4)
                                .y_grid_line_style(GridLineStyle::Dashed)
                                .y_value_label_formatter(|v| {
                                    let n = v.as_number_f32().unwrap_or(0.0);
                                    format!("${:.0}", n).into()
                                }),
                        )
                        .gridlines(GridlineConfig::horizontal())
                        .title("Dashed gridlines + $ formatter"),
                )
                .child(
                    Chart::new(sales.clone())
                        .id("hidden-ticks-chart")
                        .chart_type(ChartType::Bar)
                        .size(px(320.0), px(160.0))
                        .axis(
                            AxisConfig::new()
                                .y_tick_style(AxisTickStyle::Hidden)
                                .x_labels(vec!["Mon", "Tue", "Wed", "Thu", "Fri"]),
                        )
                        .title("Hidden Y ticks"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Legend positioning"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Mirror Swift Charts' .chartLegend(position:) — \
                             Top/Bottom share the chart's vertical axis while \
                             Leading/Trailing stack swatches beside the plot.",
                        ),
                )
                .child(
                    Chart::new(multi.clone())
                        .id("legend-top-chart")
                        .chart_type(ChartType::Bar)
                        .size(px(320.0), px(160.0))
                        .legend_position(LegendPosition::Top)
                        .title("Top legend"),
                )
                .child(
                    Chart::new(multi.clone())
                        .id("legend-trailing-chart")
                        .chart_type(ChartType::Line)
                        .size(px(320.0), px(160.0))
                        .legend_position(LegendPosition::Trailing)
                        .title("Trailing legend"),
                )
                .child(
                    Chart::new(multi.clone())
                        .id("legend-hidden-chart")
                        .chart_type(ChartType::Bar)
                        .size(px(320.0), px(160.0))
                        .legend_position(LegendPosition::Hidden)
                        .title("Hidden legend (multi-series)"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Scrolling & zoom"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Mirror Swift Charts' .chartXVisibleDomain + \
                             .chartScrollPosition — a narrow visible window \
                             filters the dataset so the plot zooms in on a \
                             slice. Use ChartView for interactive scroll-wheel \
                             panning.",
                        ),
                )
                .child({
                    let values: Vec<f32> = (0..100)
                        .map(|i| (i as f32 * 0.37).sin() * 50.0 + 60.0)
                        .collect();
                    Chart::new(ChartDataSeries::new("Signal", values))
                        .id("scroll-window-chart")
                        .chart_type(ChartType::Line)
                        .size(px(320.0), px(160.0))
                        .scroll(ChartScrollConfig::new().x_visible_domain(0.0, 9.0))
                        .title("Visible domain (first 10 of 100 points)")
                })
                .child({
                    let values: Vec<f32> = (0..100)
                        .map(|i| (i as f32 * 0.37).sin() * 50.0 + 60.0)
                        .collect();
                    Chart::new(ChartDataSeries::new("Signal", values))
                        .id("scroll-offset-chart")
                        .chart_type(ChartType::Line)
                        .size(px(320.0), px(160.0))
                        .scroll(
                            ChartScrollConfig::new()
                                .x_visible_domain(0.0, 9.0)
                                .x_scroll_position(40.0),
                        )
                        .title("Scroll position anchored at 40")
                })
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Audio Graphs accessibility"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "Attach a ChartDescriptor to expose title, summary, \
                             axes, and series to VoiceOver per Apple's \
                             AXChartDescriptor API. The summary becomes the \
                             chart's default VoiceOver label, and hosts can \
                             call ChartView::play_audio_graph to sonify the \
                             data.",
                        ),
                )
                .child({
                    let sales_points = [
                        (0.0, 10.0),
                        (1.0, 20.0),
                        (2.0, 15.0),
                        (3.0, 30.0),
                        (4.0, 25.0),
                    ];
                    let desc = ChartDescriptor::new(
                        "Quarterly sales",
                        "Sales rose from $10 in Q1 to a peak of $30 in Q4, \
                         then eased to $25 in Q5.",
                        AxisDescriptor::new("Quarter", (0.0, 4.0)),
                        AxisDescriptor::new("Sales (USD)", (0.0, 30.0)).unit("dollars"),
                    )
                    .series(vec![SeriesDescriptor::new("Sales", sales_points)]);
                    Chart::new(ChartDataSeries::new(
                        "Sales",
                        vec![10.0, 20.0, 15.0, 30.0, 25.0],
                    ))
                    .id("audio-graph-chart")
                    .chart_type(ChartType::Bar)
                    .size(px(320.0), px(160.0))
                    .audio_graph(desc)
                    .title("Quarterly sales (with Audio Graphs descriptor)")
                })
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
