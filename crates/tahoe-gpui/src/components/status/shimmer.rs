//! Shimmer/skeleton loading animation and text shimmer effects.
//!
//! Two components are provided:
//!
//! - [`Shimmer`] — a pulsing skeleton placeholder for loading states.
//! - [`TextShimmer`] / [`TextShimmerState`] — an animated text sweep effect
//!   equivalent to the AI SDK Elements `<Shimmer>` component. Sweeps
//!   left-to-right by default with smooth hermite easing (configurable via
//!   [`SweepDirection`] and [`ShimmerEasing`]).

use std::time::{Duration, Instant};

use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{
    Animation, AnimationExt, App, Context, ElementId, Entity, FontWeight, Hsla, Length, Pixels,
    SharedString, Window, div,
};

// ─── Enums ──────────────────────────────────────────────────────────────────

/// Sweep direction for the text shimmer highlight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SweepDirection {
    /// Highlight sweeps left-to-right (matches AI SDK Elements reference).
    #[default]
    LeftToRight,
    /// Highlight sweeps right-to-left.
    RightToLeft,
}

/// Easing function for the text shimmer color interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShimmerEasing {
    /// Linear interpolation — use for exact AI SDK Elements parity.
    Linear,
    /// Smooth hermite interpolation — `t²(3 − 2t)`. Default; diverges from AI SDK Elements reference.
    #[default]
    Smooth,
}

// ─── Shimmer (skeleton placeholder) ─────────────────────────────────────────

/// A pulsing skeleton placeholder that indicates loading.
#[derive(IntoElement)]
pub struct Shimmer {
    id: ElementId,
    width: Option<Length>,
    height: Option<Length>,
    label: Option<SharedString>,
}

impl Shimmer {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            width: None,
            height: None,
            label: None,
        }
    }

    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = Some(w.into());
        self
    }

    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = Some(h.into());
        self
    }

    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }
}

impl RenderOnce for Shimmer {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let duration_ms =
            crate::foundations::materials::effective_duration(theme, theme.shimmer_duration_ms);

        let anim_id = self.id.clone();
        let bg = theme.semantic.quaternary_system_fill;
        let mut el = div().id(self.id).rounded(theme.radius_md).bg(bg);

        if let Some(w) = self.width {
            el = el.w(w);
        }
        if let Some(h) = self.height {
            el = el.h(h);
        }

        if let Some(label) = self.label {
            el = el
                .flex()
                .items_center()
                .justify_center()
                .text_style(TextStyle::Subheadline, theme)
                .text_color(theme.text_muted)
                .child(label);
        }

        // HIG Accessibility: under INCREASE_CONTRAST the animated fill drops
        // below the 3:1 contrast ratio required for large non-text UI
        // elements (WCAG 1.4.11). Render the skeleton at full opacity with
        // a solid border and skip the pulse entirely so the placeholder is
        // unambiguous.
        if theme.accessibility_mode.increase_contrast() {
            return el
                .opacity(1.0)
                .border_1()
                .border_color(theme.glass.accessibility.high_contrast_border)
                .into_any_element();
        }

        // Guard: zero-duration repeating animation could tight-loop. Under
        // REDUCE_MOTION we render a static skeleton at full opacity so the
        // placeholder still reserves the incoming content's layout footprint
        // without motion or contrast compromise.
        if duration_ms == 0 {
            return el.opacity(1.0).into_any_element();
        }

        let duration = Duration::from_millis(duration_ms);
        el.with_animation(anim_id, Animation::new(duration).repeat(), |el, delta| {
            // Pulsate opacity between 0.4 and 1.0
            let t = (delta * std::f32::consts::PI).sin();
            let opacity = 0.4 + 0.6 * t;
            el.opacity(opacity)
        })
        .into_any_element()
    }
}

// ─── TextShimmer ─────────────────────────────────────────────────────────────

/// Builder for an animated text shimmer effect with a sweeping highlight.
///
/// This is the GPUI equivalent of the AI SDK Elements `<Shimmer>` component.
/// The highlight sweeps left-to-right by default with smooth hermite easing.
///
/// # Defaults
///
/// - **Direction**: [`SweepDirection::LeftToRight`] (matching AI SDK Elements)
/// - **Easing**: [`ShimmerEasing::Smooth`] (GPUI refinement; use `Linear` for
///   exact reference parity)
/// - **Duration**: from `theme.shimmer_duration_ms` (2000 ms)
/// - **Spread**: from `theme.shimmer_spread` (2.0)
/// - **Colors**: `theme.text_muted` → `theme.text` (overridable)
pub struct TextShimmer {
    text: SharedString,
    duration: Option<Duration>,
    spread: Option<f32>,
    text_size: Option<Pixels>,
    font_weight: Option<FontWeight>,
    direction: SweepDirection,
    easing: ShimmerEasing,
    base_color: Option<Hsla>,
    highlight_color: Option<Hsla>,
}

impl TextShimmer {
    /// Create a new text shimmer with the given text content.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            duration: None,
            spread: None,
            text_size: None,
            font_weight: None,
            direction: SweepDirection::default(),
            easing: ShimmerEasing::default(),
            base_color: None,
            highlight_color: None,
        }
    }

    /// Override animation duration (default from `theme.shimmer_duration_ms`).
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Set spread multiplier (default from `theme.shimmer_spread`).
    /// Higher values create a wider highlight.
    pub fn spread(mut self, spread: f32) -> Self {
        self.spread = Some(spread);
        self
    }

    /// Set font size (default from theme).
    pub fn text_size(mut self, size: Pixels) -> Self {
        self.text_size = Some(size);
        self
    }

    /// Set font weight.
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = Some(weight);
        self
    }

    /// Set sweep direction (default [`SweepDirection::LeftToRight`]).
    pub fn direction(mut self, direction: SweepDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set easing function (default [`ShimmerEasing::Smooth`]).
    pub fn easing(mut self, easing: ShimmerEasing) -> Self {
        self.easing = easing;
        self
    }

    /// Override the base (dim) color (default `theme.text_muted`).
    pub fn base_color(mut self, color: Hsla) -> Self {
        self.base_color = Some(color);
        self
    }

    /// Override the highlight (bright) color (default `theme.text`).
    pub fn highlight_color(mut self, color: Hsla) -> Self {
        self.highlight_color = Some(color);
        self
    }

    /// Build the stateful entity. Call this in a parent component's render method.
    pub fn build(self, cx: &mut App) -> Entity<TextShimmerState> {
        let theme = cx.theme();
        let base_duration_ms =
            crate::foundations::materials::effective_duration(theme, theme.shimmer_duration_ms);
        let duration = self
            .duration
            .unwrap_or_else(|| Duration::from_millis(base_duration_ms));
        let spread = self.spread.unwrap_or(theme.shimmer_spread);
        let text_size = self
            .text_size
            .unwrap_or_else(|| TextStyle::Body.attrs().size);
        let base_color = self.base_color.unwrap_or(theme.text_muted);
        let highlight_color = self.highlight_color.unwrap_or(theme.text);

        let words: Vec<SharedString> = self
            .text
            .split_whitespace()
            .map(|w| SharedString::from(w.to_string()))
            .collect();

        cx.new(|_cx| TextShimmerState {
            words,
            duration,
            spread,
            text_size,
            font_weight: self.font_weight,
            base_color,
            highlight_color,
            direction: self.direction,
            easing: self.easing,
            start: Instant::now(),
        })
    }
}

/// Internal state for the text shimmer animation.
pub struct TextShimmerState {
    words: Vec<SharedString>,
    duration: Duration,
    spread: f32,
    text_size: Pixels,
    font_weight: Option<FontWeight>,
    base_color: Hsla,
    highlight_color: Hsla,
    direction: SweepDirection,
    easing: ShimmerEasing,
    start: Instant,
}

impl TextShimmerState {
    /// Update the shimmer text content.
    pub fn set_text(&mut self, text: &str) {
        self.words = text
            .split_whitespace()
            .map(|w| SharedString::from(w.to_string()))
            .collect();
    }
}

/// Smooth hermite interpolation: `t²(3 − 2t)`.
fn smooth_ease(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Linearly interpolate between two HSLA colors.
///
/// Uses shortest-arc interpolation for the hue channel so that, e.g.,
/// red (h=0.0) → magenta (h=0.9) travels the short 0.1 arc instead of
/// the long 0.9 arc through green.
fn lerp_color(a: Hsla, b: Hsla, t: f32) -> Hsla {
    // Shortest-arc hue interpolation: pick the direction that travels
    // less than half the circle (0.5 in normalized hue space).
    let mut dh = b.h - a.h;
    if dh > 0.5 {
        dh -= 1.0;
    } else if dh < -0.5 {
        dh += 1.0;
    }
    let h = (a.h + dh * t).rem_euclid(1.0);

    Hsla {
        h,
        s: a.s + (b.s - a.s) * t,
        l: a.l + (b.l - a.l) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

impl Render for TextShimmerState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let word_count = self.words.len();
        if word_count == 0 {
            return div();
        }

        // ReduceMotion: skip animation, render static text in base color
        if self.duration.as_millis() == 0 {
            let mut container = div().flex().flex_wrap().gap(self.text_size * 0.3);
            for word in &self.words {
                container = container.child(
                    div()
                        .text_color(self.base_color)
                        .text_size(self.text_size)
                        .child(word.clone()),
                );
            }
            return container;
        }

        // Compute sweep position from elapsed time
        let elapsed = self.start.elapsed();
        let cycle = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        let delta = cycle % 1.0;

        // Sweep position based on direction
        let sweep_pos = match self.direction {
            SweepDirection::LeftToRight => delta,
            SweepDirection::RightToLeft => 1.0 - delta,
        };

        // How wide the highlight region is (normalized to 0..1)
        // spread=2.0 means the highlight covers ~2 words worth of width
        let spread_norm = self.spread / word_count.max(1) as f32;

        let base = self.base_color;
        let highlight = self.highlight_color;
        let text_size = self.text_size;
        let font_weight = self.font_weight;
        let easing = self.easing;

        // Schedule continuous re-render for animation. The `with_animation`
        // primitive used by `Shimmer` isn't directly compatible here because
        // each word receives an interpolated color and position must be
        // computed in render; keeping `cx.notify()` mirrors GPUI's text
        // animation examples.
        cx.notify();

        let theme = cx.theme();
        let mut container = div().flex().flex_wrap().gap(theme.spacing_xs);

        for (i, word) in self.words.iter().enumerate() {
            // Normalized center position of this word
            let word_pos = (i as f32 + 0.5) / word_count as f32;
            let distance = (word_pos - sweep_pos).abs();

            // Compute interpolation factor: 1.0 at sweep center, 0.0 at edge
            let t = (1.0 - distance / spread_norm.max(0.01)).clamp(0.0, 1.0);

            // Apply easing
            let t_eased = match easing {
                ShimmerEasing::Linear => t,
                ShimmerEasing::Smooth => smooth_ease(t),
            };

            let color = lerp_color(base, highlight, t_eased);

            let mut word_el = div()
                .text_color(color)
                .text_size(text_size)
                .child(word.clone());

            if let Some(weight) = font_weight {
                word_el = word_el.font_weight(weight);
            }

            container = container.child(word_el);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::{
        ShimmerEasing, SweepDirection, TextShimmer, TextShimmerState, lerp_color, smooth_ease,
    };
    use gpui::Hsla;
    use gpui::SharedString;
    use std::time::{Duration, Instant};

    fn color(h: f32, s: f32, l: f32, a: f32) -> Hsla {
        Hsla { h, s, l, a }
    }

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    // ── lerp_color ──────────────────────────────────────────────────────

    #[test]
    fn lerp_color_at_zero_returns_first() {
        let a = color(0.0, 0.5, 0.3, 1.0);
        let b = color(1.0, 1.0, 1.0, 0.0);
        let result = lerp_color(a, b, 0.0);
        assert!(approx_eq(result.h, a.h));
        assert!(approx_eq(result.s, a.s));
        assert!(approx_eq(result.l, a.l));
        assert!(approx_eq(result.a, a.a));
    }

    #[test]
    fn lerp_color_at_one_returns_second() {
        let a = color(0.1, 0.5, 0.3, 1.0);
        let b = color(0.7, 1.0, 1.0, 0.0);
        let result = lerp_color(a, b, 1.0);
        assert!(approx_eq(result.h, b.h));
        assert!(approx_eq(result.s, b.s));
        assert!(approx_eq(result.l, b.l));
        assert!(approx_eq(result.a, b.a));
    }

    #[test]
    fn lerp_color_at_midpoint() {
        // h=0.2 to h=0.6: short arc is +0.4, midpoint at 0.4
        let a = color(0.2, 0.0, 0.0, 0.0);
        let b = color(0.6, 1.0, 1.0, 1.0);
        let result = lerp_color(a, b, 0.5);
        assert!(approx_eq(result.h, 0.4));
        assert!(approx_eq(result.s, 0.5));
        assert!(approx_eq(result.l, 0.5));
        assert!(approx_eq(result.a, 0.5));
    }

    // ── enum defaults ───────────────────────────────────────────────────

    #[test]
    fn sweep_direction_default_is_left_to_right() {
        assert_eq!(SweepDirection::default(), SweepDirection::LeftToRight);
    }

    #[test]
    fn shimmer_easing_default_is_smooth() {
        assert_eq!(ShimmerEasing::default(), ShimmerEasing::Smooth);
    }

    // ── TextShimmer builder ─────────────────────────────────────────────

    #[test]
    fn text_shimmer_builder_defaults() {
        let ts = TextShimmer::new("hello world");
        assert!(ts.duration.is_none());
        assert!(ts.spread.is_none());
        assert!(ts.text_size.is_none());
        assert!(ts.font_weight.is_none());
        assert!(ts.base_color.is_none());
        assert!(ts.highlight_color.is_none());
        assert_eq!(ts.direction, SweepDirection::LeftToRight);
        assert_eq!(ts.easing, ShimmerEasing::Smooth);
    }

    #[test]
    fn text_shimmer_builder_chaining() {
        let c1 = color(0.0, 0.0, 0.5, 1.0);
        let c2 = color(0.5, 1.0, 1.0, 1.0);
        let ts = TextShimmer::new("test")
            .duration(Duration::from_secs(3))
            .spread(4.0)
            .direction(SweepDirection::RightToLeft)
            .easing(ShimmerEasing::Linear)
            .base_color(c1)
            .highlight_color(c2);

        assert_eq!(ts.duration, Some(Duration::from_secs(3)));
        assert_eq!(ts.spread, Some(4.0));
        assert_eq!(ts.direction, SweepDirection::RightToLeft);
        assert_eq!(ts.easing, ShimmerEasing::Linear);
        assert_eq!(ts.base_color, Some(c1));
        assert_eq!(ts.highlight_color, Some(c2));
    }

    // ── set_text ─────────────────────────────────────────────────────────

    /// Helper: construct a minimal TextShimmerState for logic tests.
    fn test_state(text: &str) -> TextShimmerState {
        TextShimmerState {
            words: text
                .split_whitespace()
                .map(|w| SharedString::from(w.to_string()))
                .collect(),
            duration: Duration::from_secs(2),
            spread: 2.0,
            // Finding 14: this is a text size, not an icon size. Will become
            // `theme.text_size_for(TextStyle::Body)` once a theme is threaded
            // through; keep the literal for now.
            text_size: gpui::px(14.0),
            font_weight: None,
            base_color: color(0.0, 0.0, 0.5, 1.0),
            highlight_color: color(0.0, 0.0, 1.0, 1.0),
            direction: SweepDirection::default(),
            easing: ShimmerEasing::default(),
            start: Instant::now(),
        }
    }

    #[test]
    fn set_text_splits_words() {
        let mut state = test_state("hello world");
        assert_eq!(state.words.len(), 2);
        assert_eq!(state.words[0].as_ref(), "hello");
        assert_eq!(state.words[1].as_ref(), "world");

        state.set_text("one two three");
        assert_eq!(state.words.len(), 3);
    }

    #[test]
    fn set_text_handles_empty_string() {
        let mut state = test_state("something");
        state.set_text("");
        assert!(state.words.is_empty());
    }

    #[test]
    fn set_text_handles_multiple_spaces() {
        let mut state = test_state("");
        state.set_text("a  b   c");
        assert_eq!(state.words.len(), 3);
        assert_eq!(state.words[0].as_ref(), "a");
        assert_eq!(state.words[1].as_ref(), "b");
        assert_eq!(state.words[2].as_ref(), "c");
    }

    // u2500u2500 smooth_ease u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500

    #[test]
    fn smooth_easing_formula_correctness() {
        assert!(approx_eq(smooth_ease(0.0), 0.0));
        assert!(approx_eq(smooth_ease(1.0), 1.0));
        assert!(approx_eq(smooth_ease(0.5), 0.5));
        // Monotonicity
        assert!(smooth_ease(0.25) < smooth_ease(0.5));
        assert!(smooth_ease(0.5) < smooth_ease(0.75));
    }

    // u2500u2500 lerp_color boundary u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500

    #[test]
    fn lerp_color_extrapolates_beyond_unit_range() {
        let a = color(0.0, 0.0, 0.0, 1.0);
        let b = color(1.0, 1.0, 1.0, 1.0);
        // lerp_color does NOT clamp s/l/a — callers must clamp t themselves.
        // Hue wraps via rem_euclid so it stays in 0..1.
        let result = lerp_color(a, b, 1.5);
        assert!(result.h >= 0.0 && result.h < 1.0); // hue wraps
        assert!(approx_eq(result.s, 1.5));
        assert!(approx_eq(result.l, 1.5));
    }

    #[test]
    fn lerp_color_shortest_arc_hue() {
        // Red (h=0.0) to magenta (h=0.9): short arc is -0.1 (backward),
        // NOT +0.9 (forward through green/blue).
        let red = color(0.0, 1.0, 0.5, 1.0);
        let magenta = color(0.9, 1.0, 0.5, 1.0);
        let mid = lerp_color(red, magenta, 0.5);
        // Midpoint should be at hue = 0.95 (between 0.9 and 1.0/0.0),
        // NOT at 0.45 (which is green territory).
        assert!(
            mid.h > 0.9 || mid.h < 0.1,
            "hue {:.4} should be near 0/1 boundary, not green",
            mid.h
        );
        assert!(approx_eq(mid.h, 0.95));
    }

    #[test]
    fn lerp_color_shortest_arc_forward() {
        // h=0.2 to h=0.4: short arc is forward (+0.2), no wrapping needed.
        let a = color(0.2, 1.0, 0.5, 1.0);
        let b = color(0.4, 1.0, 0.5, 1.0);
        let mid = lerp_color(a, b, 0.5);
        assert!(approx_eq(mid.h, 0.3));
    }

    // u2500u2500 sweep direction logic u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500

    #[test]
    fn sweep_direction_position_logic() {
        let delta = 0.3_f32;
        let ltr = match SweepDirection::LeftToRight {
            SweepDirection::LeftToRight => delta,
            SweepDirection::RightToLeft => 1.0 - delta,
        };
        let rtl = match SweepDirection::RightToLeft {
            SweepDirection::LeftToRight => delta,
            SweepDirection::RightToLeft => 1.0 - delta,
        };
        assert!(approx_eq(ltr, 0.3));
        assert!(approx_eq(rtl, 0.7));
        // Symmetry: ltr + rtl = 1.0
        assert!(approx_eq(ltr + rtl, 1.0));
    }
}
