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
    ActiveTheme, FontDesign, LeadingStyle, TahoeTheme, TextStyle, TextStyledExt,
};

/// Content held by a [`TextView`] — either a plain string or rich text with
/// mixed formatting via [`StyledText`].
enum TextViewContent {
    Plain(SharedString),
    Rich(StyledText),
}

/// HIG label-level color hierarchy.
///
/// The HIG defines four levels of label importance (Labels > Secondary >
/// Tertiary > Quaternary) with a fifth quinary level added in macOS Tahoe.
/// Use [`TextView::label_level`] to apply the correct semantic color without
/// reaching into the theme tokens directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LabelLevel {
    /// Primary text — `theme.text` (semantic `label`).
    #[default]
    Primary,
    /// Secondary/supplemental text — `theme.text_muted` (semantic `secondaryLabel`).
    Secondary,
    /// Tertiary text — `theme.text_tertiary()` (semantic `tertiaryLabel`).
    Tertiary,
    /// Quaternary/watermark text — `theme.text_quaternary()` (semantic `quaternaryLabel`).
    Quaternary,
    /// Quinary text (macOS Tahoe / iOS 26) — `theme.text_quinary()` (semantic `quinaryLabel`).
    Quinary,
}

impl LabelLevel {
    /// Resolve to the theme's semantic color for this level.
    pub fn resolve(self, theme: &TahoeTheme) -> Hsla {
        match self {
            Self::Primary => theme.text,
            Self::Secondary => theme.text_muted,
            Self::Tertiary => theme.text_tertiary(),
            Self::Quaternary => theme.text_quaternary(),
            Self::Quinary => theme.text_quinary(),
        }
    }
}

/// A read-only text display view per HIG.
///
/// Shows one or more paragraphs of styled text. Text selection is off by
/// default; the [`Self::selectable`] builder stores intent so callers can opt
/// in. When wired up in the future, selection will follow the same pattern as
/// [`crate::markdown::selectable_text::SelectableText`] — GPUI does not
/// provide a built-in selection API, but the crate implements it from scratch
/// using raw mouse events, `TextLayout` hit-testing, and `window.paint_quad`,
/// matching the approach Zed's editor uses.
#[derive(IntoElement)]
pub struct TextView {
    content: TextViewContent,
    style: TextStyle,
    /// Text selection intent. See struct-level doc for the implementation path.
    selectable: bool,
    max_lines: Option<usize>,
    emphasize: bool,
    color: Option<Hsla>,
    label_level: Option<LabelLevel>,
    font_design: Option<FontDesign>,
    leading_style: LeadingStyle,
    disabled: bool,
    text_align: Option<TextAlign>,
    scroll_id: Option<ElementId>,
    readable_width: bool,
}

impl TextView {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            content: TextViewContent::Plain(text.into()),
            style: TextStyle::Body,
            selectable: false,
            max_lines: None,
            emphasize: false,
            color: None,
            label_level: None,
            font_design: None,
            leading_style: LeadingStyle::default(),
            disabled: false,
            text_align: None,
            scroll_id: None,
            readable_width: false,
        }
    }

    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// Display rich text with mixed formatting (bold spans, color runs, etc.)
    /// via GPUI's [`StyledText`].
    ///
    /// Callers needing interactive text (click/hover on spans) should wrap
    /// their `StyledText` in GPUI's `InteractiveText` before passing it,
    /// rather than relying on `TextView` to provide interactivity.
    /// `InteractiveText` requires an [`ElementId`] and its own event handlers,
    /// which would change the stateless architecture of this component.
    pub fn styled_text(mut self, text: StyledText) -> Self {
        self.content = TextViewContent::Rich(text);
        self
    }

    /// Opt into text selection. Stored but not yet enforced — see the
    /// struct-level doc for the implementation path using
    /// [`crate::markdown::selectable_text::SelectableText`].
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Clamp the rendered text to `max` lines using GPUI's native
    /// `line-clamp`. Overflowing content is hidden.
    pub fn max_lines(mut self, max: usize) -> Self {
        self.max_lines = Some(max);
        self
    }

    /// Render with the HIG "Emphasized" weight for the text style (see
    /// [`TextStyle::emphasized`]). For example `Body` emphasizes to
    /// `SEMIBOLD`, `LargeTitle` to `BOLD`, and `Headline` to `BLACK`.
    pub fn emphasize(mut self, emphasize: bool) -> Self {
        self.emphasize = emphasize;
        self
    }

    /// Override the text color (default: `theme.text`).
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the text color via the HIG label-level hierarchy.
    ///
    /// Resolves to the correct semantic color (e.g. `theme.text_muted` for
    /// [`LabelLevel::Secondary`]). If both `color()` and `label_level()` are
    /// set, the explicit `color()` value wins.
    pub fn label_level(mut self, level: LabelLevel) -> Self {
        self.label_level = Some(level);
        self
    }

    /// Override the font design (default: SF Pro). Use [`FontDesign::Serif`]
    /// for editorial content, [`FontDesign::Monospaced`] for code, or
    /// [`FontDesign::Rounded`] for a friendlier tone.
    pub fn font_design(mut self, design: FontDesign) -> Self {
        self.font_design = Some(design);
        self
    }

    /// Adjust the line-height: [`LeadingStyle::Tight`], [`Standard`](LeadingStyle::Standard),
    /// or [`Loose`](LeadingStyle::Loose).
    pub fn leading_style(mut self, style: LeadingStyle) -> Self {
        self.leading_style = style;
        self
    }

    /// Render in a disabled/muted state using `theme.text_disabled()`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Override text alignment. Defaults to GPUI's default (leading-edge).
    /// HIG: "text within a text view is aligned to the leading edge" by
    /// default, but centered or trailing alignment may be appropriate in
    /// specific contexts.
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = Some(align);
        self
    }

    /// Enable vertical scrolling when the text content is taller than the
    /// view. Requires an [`ElementId`] because GPUI tracks scroll state
    /// per-element. Follows the same pattern as
    /// [`crate::components::presentation::ScrollView`].
    ///
    /// No-op when [`Self::max_lines`] is also set — clamped content cannot
    /// scroll because it is already height-constrained by GPUI's
    /// `line_clamp`.
    pub fn scrollable(mut self, id: impl Into<ElementId>) -> Self {
        self.scroll_id = Some(id.into());
        self
    }

    /// Constrain the view to the HIG readable-content optimal width
    /// (544 pt) for comfortable long-form reading.
    pub fn readable_width(mut self) -> Self {
        self.readable_width = true;
        self
    }
}

impl RenderOnce for TextView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        if cfg!(debug_assertions) && self.selectable {
            warn_once_selectable_unimplemented(std::panic::Location::caller());
        }

        // Pick the correct TextStyledExt method based on emphasize × font_design.
        let apply_typography = |el: gpui::Div,
                                style: TextStyle,
                                theme: &crate::foundations::theme::TahoeTheme,
                                emphasize: bool,
                                font_design: Option<FontDesign>| {
            match (emphasize, font_design) {
                (false, None) => el.text_style(style, theme),
                (true, None) => el.text_style_emphasized(style, theme),
                (false, Some(design)) => el.text_style_with_design(style, design, theme),
                (true, Some(design)) => el.text_style_emphasized_with_design(style, design, theme),
            }
        };

        // Always start with a plain div. If scrolling is requested, we wrap
        // the content in an id-bearing scrollable div at the end instead.
        let mut el = div();

        el = apply_typography(el, self.style, theme, self.emphasize, self.font_design);

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

        // Text color: disabled > explicit override > label_level > default theme.text.
        let text_color = if self.disabled {
            theme.text_disabled()
        } else if let Some(color) = self.color {
            color
        } else if let Some(level) = self.label_level {
            level.resolve(theme)
        } else {
            theme.text
        };
        el = el.text_color(text_color);

        if let Some(align) = self.text_align {
            el = el.text_align(align);
        }

        if let Some(max) = self.max_lines {
            el = el.line_clamp(max);
        }

        if self.readable_width {
            el = el.max_w(gpui::px(READABLE_OPTIMAL_WIDTH));
        }

        // Accessibility role — currently a no-op in GPUI but declares intent.
        let mut a11y = AccessibilityProps::new().role(AccessibilityRole::StaticText);
        if let TextViewContent::Plain(ref text) = self.content {
            a11y = a11y.label(text.clone());
        }
        el = el.with_accessibility(&a11y);

        el = match self.content {
            TextViewContent::Plain(text) => el.child(text),
            TextViewContent::Rich(styled) => el.child(styled),
        };

        // Wrap in a scrollable container when an id is provided and no
        // line_clamp is active (clamped content cannot scroll).
        let should_scroll = self.scroll_id.is_some() && self.max_lines.is_none();
        if should_scroll {
            div()
                .id(self.scroll_id.unwrap())
                .overflow_y_scroll()
                .child(el)
                .into_any_element()
        } else {
            el.into_any_element()
        }
    }
}

/// Emits at most one stderr warning per process when `.selectable(true)` is
/// called on a `TextView`, reminding developers that selection is not yet
/// implemented.
fn warn_once_selectable_unimplemented(loc: &'static std::panic::Location<'static>) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static WARNED: AtomicBool = AtomicBool::new(false);
    if WARNED.swap(true, Ordering::Relaxed) {
        return;
    }
    eprintln!(
        "[tahoe-gpui] TextView::selectable(true) at {}:{} — text selection \
         is not yet implemented; the field is stored for future use. \
         See the struct-level doc for the implementation path. \
         (This warning fires once per process.)",
        loc.file(),
        loc.line(),
    );
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::{ElementId, StyledText, TextAlign};

    use crate::foundations::theme::{FontDesign, LeadingStyle, TextStyle};

    use super::{LabelLevel, TextView, TextViewContent};

    #[test]
    fn text_view_new() {
        let tv = TextView::new("Hello world");
        assert!(matches!(
            tv.content,
            TextViewContent::Plain(ref s) if s.as_ref() == "Hello world"
        ));
        assert_eq!(tv.style, TextStyle::Body);
        assert!(!tv.selectable);
        assert!(tv.max_lines.is_none());
        assert!(!tv.emphasize);
        assert!(tv.color.is_none());
        assert!(tv.label_level.is_none());
        assert!(tv.font_design.is_none());
        assert_eq!(tv.leading_style, LeadingStyle::Standard);
        assert!(!tv.disabled);
        assert!(tv.text_align.is_none());
        assert!(tv.scroll_id.is_none());
        assert!(!tv.readable_width);
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
    fn text_view_selectable_builder() {
        let tv = TextView::new("x").selectable(true);
        assert!(tv.selectable);
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
    fn text_view_disabled() {
        let tv = TextView::new("Off").disabled(true);
        assert!(tv.disabled);
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
        let tv = TextView::new("Long form").readable_width();
        assert!(tv.readable_width);
    }

    #[test]
    fn text_view_styled_text() {
        let styled = StyledText::new("Hello rich text");
        let tv = TextView::new("placeholder").styled_text(styled);
        assert!(matches!(tv.content, TextViewContent::Rich(_)));
    }

    #[test]
    fn text_view_label_level() {
        let tv = TextView::new("Secondary").label_level(LabelLevel::Secondary);
        assert_eq!(tv.label_level, Some(LabelLevel::Secondary));
    }

    #[test]
    fn text_view_label_level_all_variants() {
        assert_eq!(
            TextView::new("a")
                .label_level(LabelLevel::Primary)
                .label_level,
            Some(LabelLevel::Primary),
        );
        assert_eq!(
            TextView::new("b")
                .label_level(LabelLevel::Secondary)
                .label_level,
            Some(LabelLevel::Secondary),
        );
        assert_eq!(
            TextView::new("c")
                .label_level(LabelLevel::Tertiary)
                .label_level,
            Some(LabelLevel::Tertiary),
        );
        assert_eq!(
            TextView::new("d")
                .label_level(LabelLevel::Quaternary)
                .label_level,
            Some(LabelLevel::Quaternary),
        );
        assert_eq!(
            TextView::new("e")
                .label_level(LabelLevel::Quinary)
                .label_level,
            Some(LabelLevel::Quinary),
        );
    }

    #[test]
    fn text_view_color_wins_over_label_level() {
        let color = gpui::hsla(0.5, 0.8, 0.6, 1.0);
        let tv = TextView::new("x")
            .label_level(LabelLevel::Secondary)
            .color(color);
        // Both are stored; the render method resolves color > label_level.
        assert_eq!(tv.color, Some(color));
        assert_eq!(tv.label_level, Some(LabelLevel::Secondary));
    }
}
