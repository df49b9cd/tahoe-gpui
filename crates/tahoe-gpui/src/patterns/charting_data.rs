//! Charting data pattern aligned with HIG.
//!
//! Charts make quantitative comparisons scannable at a glance. HIG:
//! prefer bar/column for categorical comparisons, line for trends over
//! time, scatter for correlations; always label axes, respect Dynamic
//! Type for axis/legend labels, and never rely on colour alone to convey
//! meaning (pair colour with shape, pattern, or label).
//!
//! # See also
//!
//! - [`crate::components::content::chart`] — Chart surface (stub today,
//!   lands alongside a pure-Rust chart library).
//! - [`crate::components::status::progress_indicator::ProgressIndicator`]
//!   — for single-value progress visualisations.
//! - [`crate::components::status::gauge::Gauge`] — for bounded-range
//!   measurements.
//! - [`crate::components::content::badge::Badge`] — for categorical
//!   annotations next to a chart.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/charting-data>
