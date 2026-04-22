//! Read-only styled text display aligned with HIG Text views.
//!
//! Displays multiple lines of styled, non-editable text. Unlike
//! [`super::label::Label`] (single-line) or
//! [`crate::components::selection_and_input::text_field::TextField`] (editable),
//! `TextView` is for presenting blocks of formatted content.
//!
//! # Dynamic Type
//!
//! When the theme's accessibility mode reports Bold-Text / high-contrast
//! preferences, `TextView` applies the same `effective_weight` +
//! `effective_font_scale_factor` adjustments that [`TextStyledExt`] uses for
//! the rest of the design system. This keeps the text-body scale consistent
//! with sidebar / menu / button typography when the user enables an
//! accessibility text-size mode.

use gpui::prelude::*;
use gpui::{App, ElementId, Hsla, SharedString, StyledText, TextAlign, Window, div};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::layout::READABLE_OPTIMAL_WIDTH;
use crate::foundations::theme::{
    ActiveTheme, FontDesign, LabelLevel, LeadingStyle, TahoeTheme, TextStyle, TextStyledExt,
};

/// Content held by a [`TextView`] — either a plain string or rich text
/// with mixed formatting via [`StyledText`]. Rich content carries the
/// plain-text equivalent alongside the styled element so VoiceOver has
/// something to announce without callers re-supplying the text via
/// [`TextView::accessibility_label`].
enum TextViewContent {
    Plain(SharedString),
    Rich {
        text: SharedString,
        styled: StyledText,
    },
}

/// A read-only text display view per HIG.
///
/// Shows one or more paragraphs of styled text.
///
/// # Capabilities
///
/// - Content: plain [`SharedString`] or rich [`StyledText`] via
///   [`Self::styled_text`].
/// - Typography: [`Self::text_style`], [`Self::emphasize`],
///   [`Self::font_design`], [`Self::leading_style`].
/// - Layout: [`Self::max_lines`] (line-clamp),
///   [`Self::readable_width`] (544 pt cap scaled by Dynamic Type),
///   [`Self::scrollable`] (vertical scroll), [`Self::text_align`].
/// - Color: [`Self::color`] (explicit) or [`Self::label_level`] (semantic
///   HIG hierarchy).
/// - Accessibility: [`Self::accessibility_label`] override.
///
/// # Color precedence
///
/// `color()` > `label_level()` > default `theme.text`. An explicit
/// `color()` wins over any semantic tier. For a disabled look, pass
/// `color(theme.text_disabled())` directly — `TextView` does not carry a
/// `disabled` flag because it has no interactive state.
///
/// # Layout precedence
///
/// [`Self::max_lines`] and [`Self::scrollable`] are mutually exclusive:
/// clamped content cannot scroll because its height is bounded by GPUI's
/// `line_clamp`. Setting both trips a `debug_assert!` so the conflict
/// panics in tests and debug builds; release builds silently prefer
/// `max_lines`.
///
/// # Keyboard accessibility
///
/// [`Self::scrollable`] wraps the view in a non-focusable
/// `overflow_y_scroll` container. Keyboard-only users cannot drive the
/// scroll directly because `TextView` is a stateless `RenderOnce` without
/// a [`gpui::FocusHandle`] — if keyboard scrolling matters, nest the
/// `TextView` inside a focused scroll container owned by the host app.
#[derive(IntoElement)]
pub struct TextView {
    content: TextViewContent,
    style: TextStyle,
    max_lines: Option<usize>,
    emphasize: bool,
    color: Option<Hsla>,
    label_level: Option<LabelLevel>,
    font_design: Option<FontDesign>,
    leading_style: LeadingStyle,
    text_align: Option<TextAlign>,
    scroll_id: Option<ElementId>,
    readable_width: bool,
    accessibility_label: Option<SharedString>,
}

impl TextView {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            content: TextViewContent::Plain(text.into()),
            style: TextStyle::Body,
            max_lines: None,
            emphasize: false,
            color: None,
            label_level: None,
            font_design: None,
            leading_style: LeadingStyle::default(),
            text_align: None,
            scroll_id: None,
            readable_width: false,
            accessibility_label: None,
        }
    }

    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// Display rich text with mixed formatting (bold spans, color runs,
    /// etc.) via GPUI's [`StyledText`]. `text` is the plain-text
    /// equivalent and is used as the VoiceOver label when
    /// [`Self::accessibility_label`] is not set — this keeps rich content
    /// accessible without forcing callers to restate the text twice.
    ///
    /// Callers needing interactive text (click/hover on spans) should wrap
    /// their `StyledText` in GPUI's `InteractiveText` before passing it —
    /// `InteractiveText` requires an [`ElementId`] and its own event
    /// handlers, which would change the stateless architecture of this
    /// component.
    pub fn styled_text(mut self, text: impl Into<SharedString>, styled: StyledText) -> Self {
        self.content = TextViewContent::Rich {
            text: text.into(),
            styled,
        };
        self
    }

    /// Clamp the rendered text to `max` lines using GPUI's native
    /// `line-clamp`. Overflowing content is hidden.
    ///
    /// `max_lines(0)` is ignored: `line_clamp(0)` would hide every line,
    /// which is almost never what a caller building the value dynamically
    /// wants. Pass `max_lines(1)` to keep a single line.
    pub fn max_lines(mut self, max: usize) -> Self {
        if max > 0 {
            self.max_lines = Some(max);
        }
        self
    }

    /// Render with the HIG "Emphasized" weight for the text style (see
    /// [`TextStyle::emphasized`]). For example `Body` emphasizes to
    /// `SEMIBOLD`, `LargeTitle` to `BOLD`, and `Headline` to `BLACK`.
    pub fn emphasize(mut self, emphasize: bool) -> Self {
        self.emphasize = emphasize;
        self
    }

    /// Override the text color (default: `theme.text`). Wins over
    /// [`Self::label_level`].
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the text color via the HIG label-level hierarchy.
    ///
    /// Resolves to the correct semantic color (e.g. `theme.text_muted`
    /// for [`LabelLevel::Secondary`]). If both `color()` and
    /// `label_level()` are set, the explicit `color()` value wins.
    pub fn label_level(mut self, level: LabelLevel) -> Self {
        self.label_level = Some(level);
        self
    }

    /// Override the font design (default: SF Pro). Use
    /// [`FontDesign::Serif`] for editorial content,
    /// [`FontDesign::Monospaced`] for code, or [`FontDesign::Rounded`]
    /// for a friendlier tone.
    pub fn font_design(mut self, design: FontDesign) -> Self {
        self.font_design = Some(design);
        self
    }

    /// Adjust the line-height: [`LeadingStyle::Tight`],
    /// [`Standard`](LeadingStyle::Standard), or
    /// [`Loose`](LeadingStyle::Loose).
    pub fn leading_style(mut self, style: LeadingStyle) -> Self {
        self.leading_style = style;
        self
    }

    /// Override text alignment. Defaults to GPUI's default (leading-edge).
    /// HIG: "text within a text view is aligned to the leading edge" by
    /// default, but centered or trailing alignment may be appropriate in
    /// specific contexts.
    ///
    /// Prefer leading alignment for running paragraphs — centered or
    /// right-aligned body copy breaks scanning rhythm. Reserve
    /// [`TextAlign::Center`] for short decorative labels (a single
    /// headline over a hero image) and [`TextAlign::Right`] for tabular
    /// right-aligned numerics.
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = Some(align);
        self
    }

    /// Enable vertical scrolling when the text content is taller than the
    /// view. Requires an [`ElementId`] because GPUI tracks scroll state
    /// per-element.
    ///
    /// Must not be combined with [`Self::max_lines`] — clamped content
    /// cannot scroll because its height is already bounded by GPUI's
    /// `line_clamp`. Combining the two trips a `debug_assert!` so the
    /// conflict is caught in tests; release builds silently prefer
    /// `max_lines`.
    ///
    /// # Keyboard limitation
    ///
    /// The scroll container is not focusable — see the struct-level doc.
    pub fn scrollable(mut self, id: impl Into<ElementId>) -> Self {
        self.scroll_id = Some(id.into());
        self
    }

    /// Constrain the view to the HIG readable-content optimal width
    /// ([`READABLE_OPTIMAL_WIDTH`], 544 pt) for comfortable long-form
    /// reading. Scales with Dynamic Type via
    /// [`TahoeTheme::effective_font_scale_factor`] so the column widens
    /// proportionally when the user enables a Larger Text accessibility
    /// mode.
    pub fn readable_width(mut self, readable: bool) -> Self {
        self.readable_width = readable;
        self
    }

    /// Override the VoiceOver label. Defaults to the plain-text content
    /// for both [`Self::new`] and [`Self::styled_text`] views — rich
    /// content carries its plain-text equivalent alongside the styled
    /// element.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }
}

/// Resolve the final text color from the three inputs.
///
/// Precedence (first set wins): explicit [`TextView::color`] > semantic
/// [`LabelLevel`] > default `theme.text`.
fn resolve_color(color: Option<Hsla>, level: Option<LabelLevel>, theme: &TahoeTheme) -> Hsla {
    if let Some(color) = color {
        color
    } else if let Some(level) = level {
        level.resolve(theme)
    } else {
        theme.text
    }
}

fn apply_typography(
    el: gpui::Div,
    style: TextStyle,
    theme: &TahoeTheme,
    emphasize: bool,
    font_design: Option<FontDesign>,
) -> gpui::Div {
    match (emphasize, font_design) {
        (false, None) => el.text_style(style, theme),
        (true, None) => el.text_style_emphasized(style, theme),
        (false, Some(design)) => el.text_style_with_design(style, design, theme),
        (true, Some(design)) => el.text_style_emphasized_with_design(style, design, theme),
    }
}

impl RenderOnce for TextView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // max_lines + scrollable is undefined: clamped height short-circuits
        // the scroll viewport. Assert in debug so the conflict is caught in
        // tests; release silently prefers max_lines.
        debug_assert!(
            !(self.max_lines.is_some() && self.scroll_id.is_some()),
            "TextView: max_lines() and scrollable() are mutually exclusive — \
             clamped content cannot scroll. Drop one of the two.",
        );

        let mut el = apply_typography(div(), self.style, theme, self.emphasize, self.font_design);

        // Only override line_height when leading_style differs from Standard.
        // apply_typography already sets the correct scaled leading for the
        // default case. When Tight or Loose is active, the adjusted value
        // must be scaled by effective_font_scale_factor() to match.
        if self.leading_style != LeadingStyle::Standard {
            let base_attrs = if self.emphasize {
                self.style.emphasized()
            } else {
                self.style.attrs()
            };
            let attrs = base_attrs.with_leading(self.leading_style);
            let scale = theme.effective_font_scale_factor();
            el = el.line_height(gpui::px(f32::from(attrs.leading) * scale));
        }

        el = el.text_color(resolve_color(self.color, self.label_level, theme));

        if let Some(align) = self.text_align {
            el = el.text_align(align);
        }

        if let Some(max) = self.max_lines {
            el = el.line_clamp(max);
        }

        if self.readable_width {
            // Scale the optimal width by Dynamic Type so Larger-Text
            // accessibility modes keep ~65 characters per line.
            let scale = theme.effective_font_scale_factor();
            el = el.max_w(gpui::px(READABLE_OPTIMAL_WIDTH * scale));
        }

        // A11y label falls back to plain-text (both content variants carry one).
        let label = self
            .accessibility_label
            .clone()
            .unwrap_or_else(|| match &self.content {
                TextViewContent::Plain(text) => text.clone(),
                TextViewContent::Rich { text, .. } => text.clone(),
            });
        let a11y = AccessibilityProps::new()
            .role(AccessibilityRole::StaticText)
            .label(label);

        let el = match self.content {
            TextViewContent::Plain(text) => el.child(text),
            TextViewContent::Rich { styled, .. } => el.child(styled),
        };
        let el = el.with_accessibility(&a11y);

        match self.scroll_id.filter(|_| self.max_lines.is_none()) {
            Some(id) => div()
                .id(id)
                .overflow_y_scroll()
                .child(el)
                .into_any_element(),
            None => el.into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::{ElementId, StyledText, TextAlign};

    use crate::foundations::theme::{FontDesign, LabelLevel, LeadingStyle, TahoeTheme, TextStyle};

    use super::{TextView, TextViewContent, resolve_color};

    #[test]
    fn text_view_new() {
        let tv = TextView::new("Hello world");
        assert!(matches!(
            tv.content,
            TextViewContent::Plain(ref s) if s.as_ref() == "Hello world"
        ));
        assert_eq!(tv.style, TextStyle::Body);
        assert!(tv.max_lines.is_none());
        assert!(!tv.emphasize);
        assert!(tv.color.is_none());
        assert!(tv.label_level.is_none());
        assert!(tv.font_design.is_none());
        assert_eq!(tv.leading_style, LeadingStyle::Standard);
        assert!(tv.text_align.is_none());
        assert!(tv.scroll_id.is_none());
        assert!(!tv.readable_width);
        assert!(tv.accessibility_label.is_none());
    }

    #[test]
    fn text_view_with_style() {
        let tv = TextView::new("Title").text_style(TextStyle::LargeTitle);
        assert_eq!(tv.style, TextStyle::LargeTitle);
    }

    #[test]
    fn text_view_max_lines() {
        let tv = TextView::new("Long text").max_lines(3);
        assert_eq!(tv.max_lines, Some(3));
    }

    #[test]
    fn text_view_max_lines_zero_is_ignored() {
        // line_clamp(0) would hide every line — almost never what a
        // caller computing the value dynamically wants. The builder
        // silently drops a zero so callers do not accidentally erase
        // their text.
        let tv = TextView::new("keep me").max_lines(0);
        assert_eq!(tv.max_lines, None);
    }

    #[test]
    fn text_view_emphasize() {
        let tv = TextView::new("Em").emphasize(true);
        assert!(tv.emphasize);
    }

    #[test]
    fn text_view_color_override() {
        let color = gpui::hsla(0.5, 0.8, 0.6, 1.0);
        let tv = TextView::new("Colored").color(color);
        assert_eq!(tv.color, Some(color));
    }

    #[test]
    fn text_view_font_design() {
        let tv = TextView::new("Mono").font_design(FontDesign::Monospaced);
        assert_eq!(tv.font_design, Some(FontDesign::Monospaced));
    }

    #[test]
    fn text_view_leading_style() {
        let tv = TextView::new("Tight").leading_style(LeadingStyle::Tight);
        assert_eq!(tv.leading_style, LeadingStyle::Tight);
    }

    #[test]
    fn text_view_text_align() {
        let tv = TextView::new("Center").text_align(TextAlign::Center);
        assert_eq!(tv.text_align, Some(TextAlign::Center));
    }

    #[test]
    fn text_view_scrollable() {
        let tv = TextView::new("Scroll").scrollable("scroll-id");
        assert!(tv.scroll_id.is_some());
        let id = tv.scroll_id.unwrap();
        assert!(matches!(id, ElementId::Name(n) if n == "scroll-id"));
    }

    #[test]
    fn text_view_readable_width() {
        let tv = TextView::new("Long form").readable_width(true);
        assert!(tv.readable_width);
    }

    #[test]
    fn text_view_readable_width_can_be_disabled() {
        let tv = TextView::new("Long form")
            .readable_width(true)
            .readable_width(false);
        assert!(!tv.readable_width);
    }

    #[test]
    fn text_view_styled_text_stores_plain_text() {
        let styled = StyledText::new("Bold hello");
        let tv = TextView::new("placeholder").styled_text("Bold hello", styled);
        match &tv.content {
            TextViewContent::Rich { text, .. } => assert_eq!(text.as_ref(), "Bold hello"),
            TextViewContent::Plain(_) => panic!("expected Rich content"),
        }
    }

    #[test]
    fn text_view_label_level() {
        let tv = TextView::new("Secondary").label_level(LabelLevel::Secondary);
        assert_eq!(tv.label_level, Some(LabelLevel::Secondary));
    }

    #[test]
    fn text_view_accessibility_label_builder() {
        let tv = TextView::new("visible").accessibility_label("alt");
        assert_eq!(
            tv.accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("alt"),
        );
    }

    #[test]
    fn resolve_color_defaults_to_theme_text() {
        let theme = TahoeTheme::dark();
        assert_eq!(resolve_color(None, None, &theme), theme.text);
    }

    #[test]
    fn resolve_color_label_level_resolves_to_theme_tier() {
        let theme = TahoeTheme::dark();
        assert_eq!(
            resolve_color(None, Some(LabelLevel::Secondary), &theme),
            theme.text_muted,
        );
    }

    #[test]
    fn resolve_color_explicit_wins_over_label_level() {
        let theme = TahoeTheme::dark();
        let color = gpui::hsla(0.5, 0.8, 0.6, 1.0);
        assert_eq!(
            resolve_color(Some(color), Some(LabelLevel::Secondary), &theme),
            color,
            "explicit color() must win over label_level()",
        );
    }

    #[test]
    fn resolve_color_explicit_wins_over_default() {
        let theme = TahoeTheme::dark();
        let color = gpui::hsla(0.5, 0.8, 0.6, 1.0);
        assert_eq!(resolve_color(Some(color), None, &theme), color);
    }
}
