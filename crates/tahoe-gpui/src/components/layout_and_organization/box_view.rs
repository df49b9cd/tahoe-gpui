//! Bordered group container aligned with HIG Boxes.
//!
//! A box groups related interface elements with an optional title and border,
//! providing visual separation within a view.
//!
//! HIG boxes map to `NSBox` with four distinct `boxType` values:
//! `.primary` (bordered), `.secondary` (no visible border — separator only),
//! `.separator` (thin hairline), and `.custom` (caller-styled). Each variant
//! carries slightly different defaults for border, padding, and title weight.

use gpui::prelude::*;
use gpui::{AnyElement, App, SharedString, Window, div};

use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};

/// Visual style for a [`BoxView`] — mirrors `NSBox.boxType`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum BoxStyle {
    /// Default border + surface fill. Maps to `NSBox.boxType = .primary`.
    #[default]
    Primary,
    /// No visible border; a thin top separator groups the section instead.
    /// Maps to `NSBox.boxType = .secondary`.
    Secondary,
    /// The box is *itself* a hairline separator. Renders as a thin line with
    /// no content. Maps to `NSBox.boxType = .separator`.
    Separator,
    /// No chrome — callers provide their own border/padding via children.
    /// Maps to `NSBox.boxType = .custom`.
    Custom,
}

/// A bordered group container per HIG.
///
/// Groups related elements with an optional title label and a visible border.
/// Use boxes to organize settings, form sections, or related controls.
#[derive(Default, IntoElement)]
pub struct BoxView {
    title: Option<SharedString>,
    children: Vec<AnyElement>,
    style: BoxStyle,
}

impl BoxView {
    pub fn new() -> Self {
        Self {
            title: None,
            children: Vec::new(),
            style: BoxStyle::default(),
        }
    }

    /// Set the visible style (defaults to [`BoxStyle::Primary`]).
    pub fn style(mut self, style: BoxStyle) -> Self {
        self.style = style;
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for BoxView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // Separator variant: render a 1 pt hairline and stop. The `NSBox`
        // separator style is literally just a line.
        if matches!(self.style, BoxStyle::Separator) {
            return div()
                .w_full()
                .h(theme.separator_thickness)
                .bg(theme.separator_color())
                .flex_shrink_0();
        }

        let mut container = div()
            .flex()
            .flex_col()
            .gap(theme.spacing_sm)
            .p(theme.spacing_md);

        match self.style {
            BoxStyle::Primary => {
                container = container
                    .bg(theme.surface)
                    .border_1()
                    .border_color(theme.border)
                    .rounded(theme.radius_md);
            }
            BoxStyle::Secondary => {
                // Grouped-without-border: a single top separator replaces
                // the full outline. HIG: "secondary boxes don't have a
                // visible border; use a separator to imply grouping."
                container = container.border_t_1().border_color(theme.separator_color());
            }
            BoxStyle::Custom | BoxStyle::Separator => {
                // Custom: caller styles everything. Separator is handled above.
            }
        }

        // HIG: "Write a brief phrase that describes the contents." `NSBox`
        // titles render at a *small* system-font weight above the box, not at
        // headline weight. Use `Subheadline` + muted colour so the title
        // de-emphasises relative to the contents.
        if let Some(title) = self.title {
            container = container.child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text_muted)
                    .child(title),
            );
        }

        for child in self.children {
            container = container.child(child);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::{BoxStyle, BoxView};

    #[test]
    fn box_view_new() {
        let b = BoxView::new();
        assert!(b.title.is_none());
        assert!(b.children.is_empty());
        assert_eq!(b.style, BoxStyle::Primary);
    }

    #[test]
    fn box_view_with_title() {
        let b = BoxView::new().title("Settings");
        assert_eq!(b.title.as_ref().map(|s| s.as_ref()), Some("Settings"));
    }

    #[test]
    fn box_view_style_builder() {
        let b = BoxView::new().style(BoxStyle::Secondary);
        assert_eq!(b.style, BoxStyle::Secondary);
    }

    #[test]
    fn box_style_default_is_primary() {
        assert_eq!(BoxStyle::default(), BoxStyle::Primary);
    }

    #[test]
    fn box_style_variants_distinct() {
        assert_ne!(BoxStyle::Primary, BoxStyle::Secondary);
        assert_ne!(BoxStyle::Secondary, BoxStyle::Separator);
        assert_ne!(BoxStyle::Separator, BoxStyle::Custom);
    }
}
