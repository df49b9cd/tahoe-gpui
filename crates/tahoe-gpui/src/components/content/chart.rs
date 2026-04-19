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
use gpui::{App, Hsla, Pixels, SharedString, Window, div, px};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::theme::ActiveTheme;

/// Chart mark type.
///
/// Mirrors Swift Charts' `Mark` vocabulary. v1 ships `Bar` and `Line`;
/// the remaining variants reserve the surface area so future callers can
/// opt into the full mark palette without an API break.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ChartType {
    #[default]
    Bar,
    Line,
    Area,
    Point,
    Range,
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
    chart_type: ChartType,
    series: ChartDataSeries,
    width: Pixels,
    height: Pixels,
    color: Option<Hsla>,
    accessibility_label: Option<SharedString>,
}

impl Chart {
    /// Create a new chart for the given series.
    pub fn new(series: ChartDataSeries) -> Self {
        Self {
            chart_type: ChartType::default(),
            series,
            width: px(240.0),
            height: px(120.0),
            color: None,
            accessibility_label: None,
        }
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
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let bar_color = self.color.unwrap_or(theme.accent);

        let width = self.width;
        let height = self.height;

        let min = self.series.min_value().min(0.0); // anchor at zero for bar charts so small values stay visible.
        let max = self.series.max_value().max(1e-3);
        let range = (max - min).max(1e-3);

        let a11y_label: SharedString = self
            .accessibility_label
            .clone()
            .unwrap_or_else(|| SharedString::from(self.default_accessibility_label()));
        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::Group);

        let values = self.series.values.clone();

        let mut plot = div()
            .w(width)
            .h(height)
            .bg(theme.surface)
            .rounded(theme.radius_md)
            .border_1()
            .border_color(theme.border)
            .overflow_hidden()
            .with_accessibility(&a11y_props);

        if values.is_empty() {
            return plot;
        }

        match self.chart_type {
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

                for v in &values {
                    let norm = ((v - min) / range).clamp(0.0, 1.0);
                    let bar_h = f32::from(height) * norm;
                    let bar = div()
                        .w(px(bar_width))
                        .h(px(bar_h))
                        .bg(bar_color)
                        .rounded(theme.radius_sm);
                    row = row.child(bar);
                }
                plot = plot.child(row);
            }
            ChartType::Line | ChartType::Point | ChartType::Rule => {
                // Render a sparkline: point markers placed in a flex row
                // whose vertical alignment encodes the value. A true
                // connecting stroke needs canvas rendering (GPUI `canvas`)
                // which is out of scope for v1.
                let count = values.len().max(1);
                let slot_width = f32::from(width) / count as f32;
                let point_size = 4.0_f32.max(slot_width.min(10.0));

                let mut row = div().flex().flex_row().items_end().w(width).h(height);

                for v in &values {
                    let norm = ((v - min) / range).clamp(0.0, 1.0);
                    let top_offset = f32::from(height) * (1.0 - norm) - point_size / 2.0;
                    let cell = div().w(px(slot_width)).h(height).relative().child(
                        div()
                            .absolute()
                            .top(px(top_offset.max(0.0)))
                            .left(px((slot_width - point_size) / 2.0))
                            .size(px(point_size))
                            .rounded(theme.radius_full)
                            .bg(bar_color),
                    );
                    row = row.child(cell);
                }
                plot = plot.child(row);
            }
        }

        plot
    }
}

#[cfg(test)]
mod tests {
    use super::{Chart, ChartDataSeries, ChartType};
    use core::prelude::v1::test;
    use gpui::{hsla, px};

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
}
