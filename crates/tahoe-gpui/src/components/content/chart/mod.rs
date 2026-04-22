//! HIG Chart component.
//!
//! Mirrors Apple's Swift Charts `Mark` vocabulary across six mark types:
//! Bar and Point render via GPUI `div`s; Line, Area, Range, and Rule render
//! via GPUI's canvas API (`PathBuilder::stroke`/`fill` + `paint_path`), with
//! Catmull-Rom smoothing applied when a series has ≥ 3 points.
//!
//! # API surface
//!
//! - [`Chart`] — stateless `RenderOnce` primitive. Accepts either a single
//!   [`ChartDataSeries`] or a multi-series [`ChartDataSet`]. Optional
//!   [`AxisConfig`] adds Y-axis labels + X category labels; [`GridlineConfig`]
//!   paints gridlines behind the marks. Multi-series charts auto-show a
//!   legend; call [`Chart::show_legend`] to override. [`Chart::title`] and
//!   [`Chart::subtitle`] add descriptive text above the plot.
//! - [`ChartView`] — stateful wrapper that adds an interactive hover
//!   crosshair and value tooltip, plus keyboard navigation (arrows, Home/End,
//!   Escape) so hover is reachable without a pointer.
//!
//! # Accessibility
//!
//! Each chart exposes a VoiceOver summary through
//! [`Chart::accessibility_label`]. The default label covers single-series
//! (`"{type} chart: {name}, {count} values, range {min} to {max}"`) and
//! multi-series (`"{type} chart: {n} series (name1, name2, …)"`) variants.
//!
//! Per-data-point focus for Full Keyboard Access uses
//! [`AccessibilityRole::DataPoint`](crate::foundations::accessibility::AccessibilityRole::DataPoint)
//! with `posinset`/`setsize` so VoiceOver announces "row 1 of 5" structurally
//! rather than "button". The interactive tooltip carries
//! [`AccessibilityRole::Tooltip`](crate::foundations::accessibility::AccessibilityRole::Tooltip)
//! and is labelled with its current values.
//!
//! # Color independence
//!
//! When "Differentiate Without Color" is active (macOS System Settings →
//! Accessibility → Display), bars and points receive an additional non-color
//! cue (border outline). Multi-series point charts vary marker shape per
//! series so meaning is not conveyed through colour alone.
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
