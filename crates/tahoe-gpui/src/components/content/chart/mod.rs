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
//! When "Differentiate Without Color" is active (macOS System Settings →
//! Accessibility → Display), bars and points receive an additional
//! non-color cue (a subtle border outline) so meaning is not conveyed
//! through colour alone. Multi-series support (when added) will layer
//! distinct marker shapes as well as hues.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/charts>

mod accessibility;
mod marks;
mod render;
#[cfg(test)]
mod tests;
mod types;
mod view;

pub use render::Chart;
pub use types::{
    AxisConfig, ChartDataSeries, ChartDataSet, ChartSeries, ChartType, GridlineConfig,
};
pub use view::ChartView;
