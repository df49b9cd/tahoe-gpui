//! Context window usage display widget for GPUI.
//!
//! Provides a compound component system matching the TypeScript AI Elements
//! `Context` component. The main entry point is [`ContextView`], which composes
//! the sub-components internally. Sub-components are also exported for advanced
//! custom composition.

use super::model::{ModelId, ModelPricing, default_pricing};
use super::ring::ContextRing;
use super::usage::Usage;
use crate::callback_types::OnBoolRefChange;
use crate::components::presentation::popover::Popover;
use crate::components::status::progress_indicator::ProgressIndicator;
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{App, FontWeight, SharedString, Window, div, px};

/// Width of the expanded detail card.
const DETAIL_CARD_WIDTH: f32 = 260.0;

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// Formats a percentage value (0.0–1.0), omitting the decimal for whole numbers.
///
/// Values are clamped to `0.0..=1.0`. Matches
/// `Intl.NumberFormat('en-US', {maximumFractionDigits: 1, style: 'percent'})`.
pub fn format_percent(value: f32) -> String {
    let pct = value.clamp(0.0, 1.0) * 100.0;
    let rounded = (pct * 10.0).round() / 10.0;
    if (rounded - rounded.round()).abs() < 0.01 {
        format!("{}%", rounded.round() as u32)
    } else {
        format!("{:.1}%", rounded)
    }
}

/// Formats a token count with K/M/B suffixes.
///
/// Matches `Intl.NumberFormat('en-US', {notation: 'compact'})` behavior.
pub fn format_tokens(tokens: u64) -> String {
    if tokens < 1_000 {
        tokens.to_string()
    } else if tokens < 1_000_000 {
        let v = tokens as f64 / 1_000.0;
        if v == v.round() {
            format!("{}K", v as u64)
        } else {
            format!("{:.1}K", v)
        }
    } else if tokens < 1_000_000_000 {
        let v = tokens as f64 / 1_000_000.0;
        if v == v.round() {
            format!("{}M", v as u64)
        } else {
            format!("{:.1}M", v)
        }
    } else {
        let v = tokens as f64 / 1_000_000_000.0;
        if v == v.round() {
            format!("{}B", v as u64)
        } else {
            format!("{:.1}B", v)
        }
    }
}

/// Formats a cost value as "$X.XX" or "$X.XXXX" for small amounts.
pub fn format_cost(cost: f64) -> String {
    if cost < 0.01 {
        format!("${:.4}", cost)
    } else {
        format!("${:.2}", cost)
    }
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

/// Trigger button showing the context usage percentage and a progress ring.
///
/// Matches the TypeScript `ContextTrigger` component.
#[derive(IntoElement)]
pub struct ContextPill {
    percentage: f32,
}

impl ContextPill {
    /// Creates a pill for the given usage percentage (0.0–1.0).
    pub fn new(percentage: f32) -> Self {
        Self {
            percentage: percentage.clamp(0.0, 1.0),
        }
    }
}

impl RenderOnce for ContextPill {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let pct_text = format_percent(self.percentage);
        // Button role so VoiceOver announces "button" after the label,
        // and the label describes the interactive surface the user is
        // focusing (not merely the percentage shown inside).
        let a11y = AccessibilityProps::new()
            .label(format!("Context window {pct_text}, show details"))
            .role(AccessibilityRole::Button)
            .value(pct_text.clone());

        div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .px(theme.spacing_sm)
            .py(theme.spacing_xs)
            .rounded(theme.radius_md)
            .cursor_pointer()
            .hover(|s| s.bg(theme.hover))
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                    .text_color(theme.text_muted)
                    .child(pct_text),
            )
            .child(ContextRing::new(self.percentage))
            .with_accessibility(&a11y)
    }
}

/// Header section of the context detail card.
///
/// Shows percentage, "used / total" token summary, and a progress bar.
/// Matches the TypeScript `ContextContentHeader` component.
#[derive(IntoElement)]
pub struct ContextDetailHeader {
    percentage: f32,
    used_tokens: u64,
    max_tokens: u64,
}

impl ContextDetailHeader {
    pub fn new(percentage: f32, used_tokens: u64, max_tokens: u64) -> Self {
        Self {
            percentage: percentage.clamp(0.0, 1.0),
            used_tokens,
            max_tokens,
        }
    }
}

impl RenderOnce for ContextDetailHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let token_summary = format!(
            "{} / {}",
            format_tokens(self.used_tokens),
            format_tokens(self.max_tokens),
        );

        div()
            .flex()
            .flex_col()
            .gap(theme.spacing_sm)
            .p(theme.spacing_md)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(theme.spacing_md)
                    .child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.text)
                            .child(format_percent(self.percentage)),
                    )
                    .child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.text_muted)
                            .child(token_summary),
                    ),
            )
            .child(ProgressIndicator::new(self.percentage).track_color(theme.border))
    }
}

/// A single token usage row in the detail card body.
///
/// Shows a label on the left and tokens (with optional cost) on the right.
/// Matches the TypeScript `ContextInputUsage` / `ContextOutputUsage` /
/// `ContextReasoningUsage` / `ContextCacheUsage` components.
#[derive(IntoElement)]
pub struct ContextUsageRow {
    label: SharedString,
    tokens: u64,
    cost: Option<f64>,
}

impl ContextUsageRow {
    pub fn new(label: impl Into<SharedString>, tokens: u64, cost: Option<f64>) -> Self {
        Self {
            label: label.into(),
            tokens,
            cost,
        }
    }
}

impl RenderOnce for ContextUsageRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut value_el = div().flex().items_center().gap(theme.spacing_sm);

        value_el = value_el.child(
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text)
                .child(format_tokens(self.tokens)),
        );

        if let Some(c) = self.cost
            && c > 0.0
        {
            value_el = value_el.child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .child(format!("\u{2022} {}", format_cost(c))),
            );
        }

        div()
            .flex()
            .items_center()
            .justify_between()
            .py(theme.spacing_xs)
            .child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .child(self.label),
            )
            .child(value_el)
    }
}

/// Footer section showing total cost.
///
/// Matches the TypeScript `ContextContentFooter` component.
#[derive(IntoElement)]
pub struct ContextCostFooter {
    total_cost: f64,
}

impl ContextCostFooter {
    pub fn new(total_cost: f64) -> Self {
        Self { total_cost }
    }
}

impl RenderOnce for ContextCostFooter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // HIG §Color reserves hover/accent tints for interactive state
        // feedback. The footer is at rest, so use `background` (the
        // conversation base tone, a tick darker than `surface`) to
        // create a subtle recess without borrowing a state token. The
        // top border still provides the footer separator.
        div()
            .flex()
            .flex_col()
            .child(div().w_full().h(px(1.0)).bg(theme.border))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(theme.spacing_md)
                    .p(theme.spacing_md)
                    .bg(theme.background)
                    .child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.text_muted)
                            .child("Total cost"),
                    )
                    .child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.text)
                            .child(format_cost(self.total_cost)),
                    ),
            )
    }
}

// ---------------------------------------------------------------------------
// ContextView (main compound component)
// ---------------------------------------------------------------------------

/// A widget displaying AI context window usage.
///
/// Composes [`ContextPill`], [`ContextDetailHeader`], [`ContextUsageRow`],
/// and [`ContextCostFooter`] into a hover card. Matches the TypeScript
/// AI Elements `Context` compound component.
///
/// # Example
///
/// ```ignore
/// ContextView::new()
///     .model_id(&ModelId::parse("anthropic:claude-sonnet-4"))
///     .max_tokens(200_000)
///     .used_tokens(42_000)
///     .usage(Usage::new(30_000, 10_000, 0, 2_000))
///     .expanded(true)
///     .on_hover_change(|&hovered, _window, _cx| {
///         // parent flips its expanded bool
///     })
/// ```
#[derive(IntoElement)]
pub struct ContextView {
    max_tokens: u64,
    used_tokens: u64,
    usage: Usage,
    model_id: ModelId,
    pricing: Option<ModelPricing>,
    expanded: bool,
    on_hover_change: OnBoolRefChange,
}

impl ContextView {
    /// Creates a new context view with default values.
    pub fn new() -> Self {
        Self {
            max_tokens: 0,
            used_tokens: 0,
            usage: Usage::default(),
            model_id: ModelId::parse("custom:unknown"),
            pricing: None,
            expanded: false,
            on_hover_change: None,
        }
    }

    /// Sets the maximum token capacity.
    pub fn max_tokens(mut self, v: u64) -> Self {
        self.max_tokens = v;
        self
    }

    /// Sets the number of tokens currently used.
    pub fn used_tokens(mut self, v: u64) -> Self {
        self.used_tokens = v;
        self
    }

    /// Sets the token usage breakdown.
    pub fn usage(mut self, v: Usage) -> Self {
        self.usage = v;
        self
    }

    /// Sets the model identifier and auto-populates pricing.
    pub fn model_id(mut self, v: &ModelId) -> Self {
        self.model_id = v.clone();
        if self.pricing.is_none() {
            self.pricing = Some(default_pricing(v));
        }
        self
    }

    /// Overrides the model pricing.
    pub fn pricing(mut self, v: ModelPricing) -> Self {
        self.pricing = Some(v);
        self
    }

    /// Sets whether the detail view is expanded.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Sets the hover change callback, fired with `true` on mouse enter and `false` on leave.
    pub fn on_hover_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_change = Some(Box::new(handler));
        self
    }

    fn percentage(&self) -> f32 {
        if self.max_tokens > 0 {
            (self.used_tokens as f32 / self.max_tokens as f32).min(1.0)
        } else {
            0.0
        }
    }

    fn usage_rows(&self) -> Vec<ContextUsageRow> {
        let mut rows = Vec::new();
        if self.usage.input > 0 {
            let cost = self
                .pricing
                .map(|p| (self.usage.input as f64 / 1e6) * p.input);
            rows.push(ContextUsageRow::new("Input", self.usage.input, cost));
        }
        if self.usage.output > 0 {
            let cost = self
                .pricing
                .map(|p| (self.usage.output as f64 / 1e6) * p.output);
            rows.push(ContextUsageRow::new("Output", self.usage.output, cost));
        }
        if self.usage.reasoning > 0 {
            let cost = self
                .pricing
                .map(|p| (self.usage.reasoning as f64 / 1e6) * p.reasoning);
            rows.push(ContextUsageRow::new(
                "Reasoning",
                self.usage.reasoning,
                cost,
            ));
        }
        if self.usage.cached > 0 {
            let cost = self
                .pricing
                .map(|p| (self.usage.cached as f64 / 1e6) * p.cached);
            rows.push(ContextUsageRow::new("Cache", self.usage.cached, cost));
        }
        rows
    }
}

impl Default for ContextView {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ContextView {
    fn render(mut self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let expanded = self.expanded;
        let percentage = self.percentage();
        let on_hover_change = self.on_hover_change.take();

        // Trigger: pill button.
        let pill = ContextPill::new(percentage).into_any_element();

        // Detail card content.
        let rows = self.usage_rows();
        let total_cost = self
            .pricing
            .map(|p| p.calculate_cost(&self.usage))
            .unwrap_or(0.0);

        let mut card = div()
            .w(px(DETAIL_CARD_WIDTH))
            .bg(theme.surface)
            .rounded(theme.radius_lg)
            .border_1()
            .border_color(theme.border)
            .shadow_md()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(ContextDetailHeader::new(
                percentage,
                self.used_tokens,
                self.max_tokens,
            ));

        // Body: usage rows (with divider).
        if !rows.is_empty() {
            card = card.child(div().w_full().h(px(1.0)).bg(theme.border));
            let mut body = div().flex().flex_col().p(theme.spacing_md);
            for row in rows {
                body = body.child(row);
            }
            card = card.child(body);
        }

        // Footer: total cost (only when pricing produces a non-zero cost).
        if self.pricing.is_some() && total_cost > 0.0 {
            card = card.child(ContextCostFooter::new(total_cost));
        }

        let card = card.into_any_element();

        let mut container = div()
            .id("context-view")
            .child(Popover::new("context-popover", pill, card).visible(expanded));

        if let Some(handler) = on_hover_change {
            container = container.on_hover(handler);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::{ContextView, Usage, format_cost, format_percent, format_tokens};
    use core::prelude::v1::test;

    #[test]
    fn format_percent_whole() {
        assert_eq!(format_percent(0.0), "0%");
        assert_eq!(format_percent(0.5), "50%");
        assert_eq!(format_percent(1.0), "100%");
    }

    #[test]
    fn format_percent_fractional() {
        assert_eq!(format_percent(0.333), "33.3%");
        assert_eq!(format_percent(0.666), "66.6%");
        assert_eq!(format_percent(0.999), "99.9%");
    }

    #[test]
    fn format_tokens_small() {
        assert_eq!(format_tokens(0), "0");
        assert_eq!(format_tokens(999), "999");
    }

    #[test]
    fn format_tokens_thousands() {
        assert_eq!(format_tokens(1_000), "1K");
        assert_eq!(format_tokens(1_500), "1.5K");
        assert_eq!(format_tokens(32_000), "32K");
    }

    #[test]
    fn format_tokens_millions() {
        assert_eq!(format_tokens(1_000_000), "1M");
        assert_eq!(format_tokens(1_500_000), "1.5M");
        assert_eq!(format_tokens(128_000_000), "128M");
    }

    #[test]
    fn format_tokens_billions() {
        assert_eq!(format_tokens(1_000_000_000), "1B");
        assert_eq!(format_tokens(2_500_000_000), "2.5B");
    }

    #[test]
    fn format_cost_small() {
        assert_eq!(format_cost(0.001), "$0.0010");
    }

    #[test]
    fn format_cost_normal() {
        assert_eq!(format_cost(1.50), "$1.50");
    }

    #[test]
    fn usage_rows_assigns_correct_labels() {
        let view = ContextView::new().usage(Usage::new(100, 200, 50, 10));
        let rows = view.usage_rows();
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0].label, "Input");
        assert_eq!(rows[1].label, "Output");
        assert_eq!(rows[2].label, "Reasoning");
        assert_eq!(rows[3].label, "Cache");
    }

    #[test]
    fn usage_rows_skips_zero_types() {
        let view = ContextView::new().usage(Usage::new(100, 0, 0, 0));
        let rows = view.usage_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "Input");
    }

    #[test]
    fn percentage_zero_when_no_max() {
        let view = ContextView::new();
        assert_eq!(view.percentage(), 0.0);
    }

    #[test]
    fn percentage_capped_at_one() {
        let view = ContextView::new().max_tokens(100).used_tokens(200);
        assert_eq!(view.percentage(), 1.0);
    }
}
