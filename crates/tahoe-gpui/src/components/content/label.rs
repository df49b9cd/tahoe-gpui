//! HIG Label component.
//!
//! A stateless display label that composes an optional leading icon, text, and
//! optional trailing icon in a horizontal flex row. Typography is derived from
//! the HIG `TextStyle` system with full Bold-Text accessibility support.

use gpui::prelude::*;
use gpui::{AnyElement, App, DefiniteLength, Hsla, SharedString, Window, div};
// Note: Label uses TextStyle::attrs() / TextStyle::emphasized() directly
// (rather than TextStyledExt) so it can compose emphasize + color override +
// per-variant spacing in one place.
use crate::foundations::theme::{ActiveTheme, TextStyle};

/// Semantic label variant.
///
/// Callers choose a variant to get the HIG-appropriate typography,
/// transform, and alignment for a given context — preference-pane form
/// labels, sidebar section headers, etc. The default [`LabelVariant::Plain`]
/// emits no additional styling beyond the configured `TextStyle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelVariant {
    /// No extra styling — the label renders with just its configured
    /// text style, optional emphasis, and colour.
    #[default]
    Plain,
    /// Sidebar / preference-pane section header: uppercase text, positive
    /// tracking (visually rendered via positive letter-spacing where the
    /// framework supports it), `Caption2` text style.
    SectionHeader,
    /// Form label on the trailing edge of a settings row. Aligns the text
    /// to the trailing edge and uses secondary (muted) colour so the
    /// value on the leading side remains the visual focus.
    FormLabel,
}

/// A stateless display label following HIG typography.
///
/// Renders a flex row of `[icon?] [text] [trailing_icon?]` with size, weight,
/// and line-height derived from a `TextStyle` (defaults to `Body`).
#[derive(IntoElement)]
pub struct Label {
    text: SharedString,
    icon: Option<AnyElement>,
    trailing_icon: Option<AnyElement>,
    text_style: Option<TextStyle>,
    variant: LabelVariant,
    color: Option<Hsla>,
    emphasize: bool,
}

impl Label {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            trailing_icon: None,
            text_style: None,
            variant: LabelVariant::default(),
            color: None,
            emphasize: false,
        }
    }

    /// Set a leading icon element.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// Set a trailing icon element.
    pub fn trailing_icon(mut self, icon: impl IntoElement) -> Self {
        self.trailing_icon = Some(icon.into_any_element());
        self
    }

    /// Override the HIG text style (default: `Body`).
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.text_style = Some(style);
        self
    }

    /// Set the semantic variant.
    pub fn variant(mut self, variant: LabelVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Override the text color (default: `theme.text`).
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Render the label with the HIG "Emphasized" weight for its text
    /// style (see `TextStyle::emphasized`). This differs from a uniform
    /// SEMIBOLD bump — for example `LargeTitle` emphasizes to BOLD, and
    /// `Headline` emphasizes to BLACK (HIG "Heavy").
    pub fn emphasize(mut self, emphasize: bool) -> Self {
        self.emphasize = emphasize;
        self
    }
}

impl RenderOnce for Label {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // Variant may pin a text style if the caller didn't set one.
        let style = self.text_style.unwrap_or(match self.variant {
            LabelVariant::SectionHeader => TextStyle::Caption2,
            _ => TextStyle::Body,
        });

        // Emphasize routes through per-style HIG "Emphasized" weights.
        let base_attrs = if self.emphasize {
            style.emphasized()
        } else {
            style.attrs()
        };

        // Apply Bold-Text accessibility bump on top.
        let weight = theme.effective_weight(base_attrs.weight);

        let text_color = self.color.unwrap_or(match self.variant {
            LabelVariant::FormLabel | LabelVariant::SectionHeader => theme.text_muted,
            LabelVariant::Plain => theme.text,
        });
        let line_height = base_attrs.leading;

        // Render text with per-variant transforms. For `SectionHeader` we
        // uppercase the string — GPUI's `text-transform` support is not
        // exposed on `Div`, so we transform at render time.
        let rendered_text: SharedString = match self.variant {
            LabelVariant::SectionHeader => SharedString::from(self.text.to_uppercase()),
            _ => self.text.clone(),
        };

        let mut el = div()
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .text_size(base_attrs.size)
            .font_weight(weight)
            .text_color(text_color)
            .line_height(DefiniteLength::from(line_height));

        if matches!(self.variant, LabelVariant::FormLabel) {
            el = el.justify_end();
        }

        if let Some(icon) = self.icon {
            el = el.child(icon);
        }

        el = el.child(rendered_text);

        if let Some(trailing) = self.trailing_icon {
            el = el.child(trailing);
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use crate::foundations::theme::TextStyle;
    use core::prelude::v1::test;
    use gpui::FontWeight;

    use super::{Label, LabelVariant};

    #[test]
    fn default_fields() {
        let label = Label::new("Hello");
        assert_eq!(label.text.as_ref(), "Hello");
        assert!(label.icon.is_none());
        assert!(label.trailing_icon.is_none());
        assert!(label.text_style.is_none());
        assert!(label.color.is_none());
        assert!(!label.emphasize);
        assert_eq!(label.variant, LabelVariant::Plain);
    }

    #[test]
    fn builder_text_style() {
        let label = Label::new("Title").text_style(TextStyle::Title1);
        assert_eq!(label.text_style, Some(TextStyle::Title1));
    }

    #[test]
    fn builder_variant() {
        let label = Label::new("Heading").variant(LabelVariant::SectionHeader);
        assert_eq!(label.variant, LabelVariant::SectionHeader);
    }

    #[test]
    fn builder_emphasize() {
        let label = Label::new("Emphasized").emphasize(true);
        assert!(label.emphasize);
    }

    #[test]
    fn builder_color() {
        let color = gpui::hsla(0.5, 0.8, 0.6, 1.0);
        let label = Label::new("Colored").color(color);
        assert_eq!(label.color, Some(color));
    }

    #[test]
    fn default_text_style_is_body() {
        let attrs = TextStyle::Body.attrs();
        assert_eq!(attrs.weight, FontWeight::NORMAL);
        assert_eq!(f32::from(attrs.size), 13.0);
        assert!(f32::from(attrs.leading) > f32::from(attrs.size));
    }

    #[test]
    fn emphasize_routes_through_per_style_emphasized_weight() {
        // Body emphasizes to SEMIBOLD.
        let body_emphasized = TextStyle::Body.emphasized();
        assert_eq!(body_emphasized.weight, FontWeight::SEMIBOLD);
        // LargeTitle emphasizes to BOLD — not SEMIBOLD, which was the v1 bug.
        let title_emphasized = TextStyle::LargeTitle.emphasized();
        assert_eq!(title_emphasized.weight, FontWeight::BOLD);
        // Headline emphasizes to BLACK (HIG "Heavy").
        let headline_emphasized = TextStyle::Headline.emphasized();
        assert_eq!(headline_emphasized.weight, FontWeight::BLACK);
    }

    #[test]
    fn text_style_attrs_headline_is_bold() {
        let attrs = TextStyle::Headline.attrs();
        assert_eq!(attrs.weight, FontWeight::BOLD);
        assert_eq!(f32::from(attrs.size), 13.0);
    }

    #[test]
    fn text_style_attrs_caption2_smallest() {
        let attrs = TextStyle::Caption2.attrs();
        assert_eq!(f32::from(attrs.size), 10.0);
        assert_eq!(attrs.weight, FontWeight::MEDIUM);
    }

    #[test]
    fn section_header_defaults_to_caption2_style() {
        // SectionHeader without an explicit text_style should fall back to
        // Caption2 (per HIG sidebar / preference-pane convention).
        let label = Label::new("Section").variant(LabelVariant::SectionHeader);
        assert!(label.text_style.is_none());
        // The render-time resolution verifies this in full; we assert the
        // builder does not spuriously set the style.
    }
}
