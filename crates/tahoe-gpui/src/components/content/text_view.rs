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
use gpui::{App, SharedString, Window, div};

use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};

/// A read-only text display view per HIG.
///
/// Shows one or more paragraphs of styled text. Text selection is not yet
/// available in GPUI; the builder method exists for source-compat with
/// callers that need to opt in once upstream lands the API.
#[derive(IntoElement)]
pub struct TextView {
    text: SharedString,
    style: TextStyle,
    /// GPUI does not currently expose text selection control. The field is
    /// retained so callers can express intent today; actual selection will
    /// activate when upstream lands the API. See
    /// `docs/hig/components/content.md` — Text views for status.
    #[allow(dead_code)]
    selectable: bool,
    max_lines: Option<usize>,
}

impl TextView {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::Body,
            selectable: true,
            max_lines: None,
        }
    }

    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// Opt into text selection when/if the platform supports it. Stored
    /// but not enforced — see the module-level comment.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Clamp the rendered text to `max` lines. The container clips
    /// overflowing content; callers that need "… truncation must
    /// pre-truncate the string because GPUI does not yet expose
    /// `line-clamp`.
    pub fn max_lines(mut self, max: usize) -> Self {
        self.max_lines = Some(max);
        self
    }
}

impl RenderOnce for TextView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // `text_style` applies per-style size, weight, and line-height —
        // scaled by `theme.effective_font_scale_factor()` and bumped per
        // `theme.effective_weight()` when Bold-Text is active. Matches the
        // Dynamic-Type behaviour of the rest of the design system.
        let mut el = div().text_style(self.style, theme).text_color(theme.text);

        if let Some(max) = self.max_lines {
            // Clamp to `max * scaled leading` so the container clips any
            // overflow rather than silently ignoring `max_lines`.
            let scale = theme.effective_font_scale_factor();
            let leading = f32::from(self.style.attrs().leading) * scale;
            let height = leading * (max as f32);
            el = el.overflow_hidden().max_h(gpui::px(height));
        }

        el.child(self.text)
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use crate::foundations::theme::TextStyle;

    use super::TextView;

    #[test]
    fn text_view_new() {
        let tv = TextView::new("Hello world");
        assert_eq!(tv.text.as_ref(), "Hello world");
        assert_eq!(tv.style, TextStyle::Body);
        assert!(tv.selectable);
        assert!(tv.max_lines.is_none());
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
        let tv = TextView::new("x").selectable(false);
        assert!(!tv.selectable);
    }
}
