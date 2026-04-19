//! Context window usage display, model identification, and cost estimation.
//!
//! Ported from an earlier `iced`-based prototype. Provides:
//! - [`Usage`] — token usage breakdown (input, output, reasoning, cached)
//! - [`ModelId`] / [`Provider`] — model identification and parsing
//! - [`ModelPricing`] — per-million-token pricing with cost calculation
//! - [`ContextView`] — GPUI widget showing context window usage as a
//!   compact pill with hover-to-expand detail card

pub mod model;
pub mod ring;
pub mod usage;
pub mod widget;

pub use model::{ModelId, ModelPricing, Provider, default_pricing};
pub use ring::ContextRing;
pub use usage::Usage;
pub use widget::{
    ContextCostFooter, ContextDetailHeader, ContextPill, ContextUsageRow, ContextView, format_cost,
    format_percent, format_tokens,
};
