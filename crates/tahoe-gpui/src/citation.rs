//! Inline citation component with hover card showing source details.

use crate::components::presentation::popover::{Popover, PopoverPlacement};
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TahoeTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{
    App, ElementId, FocusHandle, Focusable, FontWeight, KeyDownEvent, SharedString, Window, div, px,
};

/// Source data for a citation hover card.
#[derive(Clone)]
pub struct CitationSource {
    pub title: Option<SharedString>,
    pub url: Option<SharedString>,
    pub snippet: Option<SharedString>,
    pub description: Option<SharedString>,
    pub quote: Option<SharedString>,
}

impl Default for CitationSource {
    fn default() -> Self {
        Self::new()
    }
}

impl CitationSource {
    pub fn new() -> Self {
        Self {
            title: None,
            url: None,
            snippet: None,
            description: None,
            quote: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn url(mut self, url: impl Into<SharedString>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn snippet(mut self, snippet: impl Into<SharedString>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn quote(mut self, quote: impl Into<SharedString>) -> Self {
        self.quote = Some(quote.into());
        self
    }

    /// Extract the hostname from the URL, if present.
    /// E.g., `"https://docs.rs/foo/bar"` → `"docs.rs"`.
    pub fn hostname(&self) -> Option<&str> {
        let url = self.url.as_ref()?.as_ref();
        let after_scheme = url.split("://").nth(1).unwrap_or(url);
        let authority_end = after_scheme
            .find(['/', '?', '#'])
            .unwrap_or(after_scheme.len());
        let authority = &after_scheme[..authority_end];
        let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
        if host.is_empty() { None } else { Some(host) }
    }
}

/// Stateful citation popover with hover card and carousel for multiple sources.
pub struct CitationPopover {
    element_id: ElementId,
    index: usize,
    sources: Vec<CitationSource>,
    is_hovered: bool,
    carousel_index: usize,
    focus_handle: FocusHandle,
}

impl CitationPopover {
    pub fn new(index: usize, sources: Vec<CitationSource>, cx: &mut Context<Self>) -> Self {
        Self {
            element_id: ElementId::Name(format!("citation-popover-{index}").into()),
            index,
            sources,
            is_hovered: false,
            carousel_index: 0,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn update_sources(&mut self, sources: Vec<CitationSource>, cx: &mut Context<Self>) {
        self.sources = sources;
        self.carousel_index = self
            .carousel_index
            .min(self.sources.len().saturating_sub(1));
        cx.notify();
    }

    fn render_source(source: &CitationSource, theme: &TahoeTheme) -> gpui::Div {
        let mut content = div()
            .p(theme.spacing_sm)
            .flex()
            .flex_col()
            .gap(theme.spacing_xs);

        if let Some(ref title) = source.title {
            content = content.child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
                    .text_color(theme.text)
                    .overflow_hidden()
                    .child(title.clone()),
            );
        }

        if let Some(ref url) = source.url {
            content = content.child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.accent)
                    .overflow_hidden()
                    .child(url.clone()),
            );
        }

        if let Some(ref description) = source.description {
            content = content.child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .max_h(px(60.0))
                    .overflow_hidden()
                    .child(description.clone()),
            );
        }

        if let Some(ref snippet) = source.snippet {
            content = content.child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .max_h(px(60.0))
                    .overflow_hidden()
                    .child(snippet.clone()),
            );
        }

        if let Some(ref quote) = source.quote {
            content = content.child(
                div()
                    .pl(theme.spacing_sm)
                    .border_l_2()
                    .border_color(theme.border)
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .max_h(px(80.0))
                    .overflow_hidden()
                    .child(quote.clone()),
            );
        }

        content
    }
}

impl Render for CitationPopover {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // Badge label
        let badge_label = if let Some(first) = self.sources.first() {
            let host = first.hostname().unwrap_or("source");
            if self.sources.len() > 1 {
                format!("{} +{}", host, self.sources.len() - 1)
            } else {
                host.to_string()
            }
        } else {
            format!("[{}]", self.index)
        };

        let first_url = self.sources.first().and_then(|s| s.url.clone());

        // Screen readers would otherwise see only the hostname glyph —
        // tag it as an interactive citation so VoiceOver reads
        // "Citation docs.rs +2, button" rather than just the text.
        let badge_a11y = AccessibilityProps::new()
            .label(format!("Citation {}", badge_label))
            .role(AccessibilityRole::Button);

        let mut badge = div()
            .id(ElementId::Name(
                format!("citation-badge-{}", self.index).into(),
            ))
            .text_style(TextStyle::Caption1, theme)
            .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
            .text_color(theme.accent)
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_full)
            .px(theme.spacing_xs)
            .child(badge_label)
            .with_accessibility(&badge_a11y);

        if let Some(url) = first_url {
            badge = badge.cursor_pointer().on_click(move |_event, _window, cx| {
                cx.open_url(url.as_ref());
            });
        }

        // Build popover content (carousel + source card)
        let mut popover_content = div().w(px(320.0)).flex().flex_col();

        // Carousel header (only for multiple sources)
        if self.sources.len() > 1 {
            let at_min = self.carousel_index == 0;
            let at_max = self.carousel_index >= self.sources.len() - 1;

            // Dimmed carousel controls otherwise appear as an opaque
            // grey rectangle to VoiceOver. Label the disabled state so
            // users know why the control isn't responsive.
            let prev_label = if at_min {
                "Previous source, unavailable"
            } else {
                "Previous source"
            };
            let next_label = if at_max {
                "Next source, unavailable"
            } else {
                "Next source"
            };
            let prev_a11y = AccessibilityProps::new()
                .label(prev_label)
                .role(AccessibilityRole::Button);
            let next_a11y = AccessibilityProps::new()
                .label(next_label)
                .role(AccessibilityRole::Button);

            let mut prev_btn = div()
                .id(ElementId::Name(
                    format!("cite-carousel-prev-{}", self.index).into(),
                ))
                .p(px(7.0))
                .rounded(theme.radius_sm)
                .child(Icon::new(IconName::ChevronLeft).size(px(10.0)))
                .with_accessibility(&prev_a11y);
            if at_min {
                prev_btn = prev_btn.opacity(0.4).cursor_default();
            } else {
                prev_btn = prev_btn.cursor_pointer().on_click(cx.listener(
                    |this, _event: &gpui::ClickEvent, _window, cx| {
                        this.carousel_index = this.carousel_index.saturating_sub(1);
                        cx.notify();
                    },
                ));
            }

            let mut next_btn = div()
                .id(ElementId::Name(
                    format!("cite-carousel-next-{}", self.index).into(),
                ))
                .p(px(7.0))
                .rounded(theme.radius_sm)
                .child(Icon::new(IconName::ChevronRight).size(px(10.0)))
                .with_accessibility(&next_a11y);
            if at_max {
                next_btn = next_btn.opacity(0.4).cursor_default();
            } else {
                next_btn = next_btn.cursor_pointer().on_click(cx.listener(
                    |this, _event: &gpui::ClickEvent, _window, cx| {
                        this.carousel_index =
                            (this.carousel_index + 1).min(this.sources.len().saturating_sub(1));
                        cx.notify();
                    },
                ));
            }

            let header = div()
                .flex()
                .items_center()
                .justify_between()
                .px(theme.spacing_sm)
                .py(theme.spacing_xs)
                .bg(theme.hover)
                .child(prev_btn)
                .child(
                    div()
                        .text_style(TextStyle::Caption1, theme)
                        .text_color(theme.text_muted)
                        .child(format!(
                            "{}/{}",
                            self.carousel_index + 1,
                            self.sources.len()
                        )),
                )
                .child(next_btn);

            popover_content = popover_content.child(header);
        }

        // Source content
        if let Some(source) = self.sources.get(self.carousel_index) {
            popover_content = popover_content.child(Self::render_source(source, theme));
        }

        let is_focused = self.focus_handle.is_focused(_window);
        let is_visible = (self.is_hovered || is_focused) && !self.sources.is_empty();

        // Compose with the Popover primitive
        let popover = Popover::new(self.element_id.clone(), badge, popover_content)
            .visible(is_visible)
            .placement(PopoverPlacement::BelowLeft)
            .with_focus_handle(self.focus_handle.clone());

        div()
            .id(ElementId::Name(
                format!("citation-outer-{}", self.index).into(),
            ))
            .track_focus(&self.focus_handle)
            .on_hover(cx.listener(|this, hovered: &bool, _window, cx| {
                this.is_hovered = *hovered;
                cx.notify();
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                match event.keystroke.key.as_str() {
                    "left" => {
                        this.carousel_index = this.carousel_index.saturating_sub(1);
                        cx.notify();
                    }
                    "right" => {
                        this.carousel_index =
                            (this.carousel_index + 1).min(this.sources.len().saturating_sub(1));
                        cx.notify();
                    }
                    "escape" => {
                        this.is_hovered = false;
                        cx.notify();
                    }
                    "enter" => {
                        if let Some(source) = this.sources.get(this.carousel_index)
                            && let Some(ref url) = source.url
                        {
                            cx.open_url(url.as_ref());
                        }
                    }
                    _ => {}
                }
            }))
            .child(popover)
    }
}

impl Focusable for CitationPopover {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

/// An inline citation badge that shows a numbered reference.
///
/// Renders as a small superscript badge `[N]` in accent color.
/// Clicking opens the source URL if available.
/// For hover cards with source details, use [`CitationPopover`] instead.
#[derive(IntoElement)]
pub struct InlineCitation {
    id: ElementId,
    index: usize,
    source: CitationSource,
}

impl InlineCitation {
    pub fn new(id: impl Into<ElementId>, index: usize) -> Self {
        Self {
            id: id.into(),
            index,
            source: CitationSource::new(),
        }
    }

    /// Set the URL for the citation (also enables click-to-open).
    pub fn url(mut self, url: impl Into<SharedString>) -> Self {
        self.source.url = Some(url.into());
        self
    }

    /// Set the source title shown in the hover card.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.source.title = Some(title.into());
        self
    }

    /// Set a text snippet shown in the hover card.
    pub fn snippet(mut self, snippet: impl Into<SharedString>) -> Self {
        self.source.snippet = Some(snippet.into());
        self
    }

    /// Set the full source data.
    pub fn source(mut self, source: CitationSource) -> Self {
        self.source = source;
        self
    }
}

impl RenderOnce for InlineCitation {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let label = format!("[{}]", self.index);
        let index = self.index;
        let has_url = self.source.url.is_some();

        // VoiceOver label describes the citation as a link when a URL is
        // attached, otherwise as a static reference marker.
        let a11y_label = match (&self.source.title, has_url) {
            (Some(title), true) => format!("Citation {index}, open {title}"),
            (None, true) => format!("Citation {index}, open source URL"),
            (Some(title), false) => format!("Citation {index}, {title}"),
            (None, false) => format!("Citation {index}"),
        };
        let a11y = AccessibilityProps::new()
            .label(a11y_label)
            .role(if has_url {
                AccessibilityRole::Button
            } else {
                AccessibilityRole::StaticText
            });

        let mut el = div()
            .id(self.id)
            .child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.accent)
                    .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
                    .child(label),
            )
            .with_accessibility(&a11y);

        // F12: keyboard reachability. When the badge has a URL, it acts
        // as a link — make it focusable so Tab navigation can land on it
        // and activate it with Return / Space, matching the popover form.
        if let Some(url) = self.source.url {
            let url_for_click = url.clone();
            el = el
                .cursor_pointer()
                .focusable()
                .on_click(move |_event, _window, cx| {
                    cx.open_url(url_for_click.as_ref());
                })
                .on_key_down(move |event: &KeyDownEvent, _window, cx| {
                    if crate::foundations::keyboard::is_activation_key(event) {
                        cx.stop_propagation();
                        cx.open_url(url.as_ref());
                    }
                });
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use super::CitationSource;
    use core::prelude::v1::test;

    #[test]
    fn hostname_standard_url() {
        let source = CitationSource::new().url("https://docs.rs/foo/bar");
        assert_eq!(source.hostname(), Some("docs.rs"));
    }

    #[test]
    fn hostname_url_without_path() {
        let source = CitationSource::new().url("https://example.com");
        assert_eq!(source.hostname(), Some("example.com"));
    }

    #[test]
    fn hostname_url_with_port() {
        let source = CitationSource::new().url("https://localhost:3000/api");
        assert_eq!(source.hostname(), Some("localhost:3000"));
    }

    #[test]
    fn hostname_no_url() {
        let source = CitationSource::new();
        assert_eq!(source.hostname(), None);
    }

    #[test]
    fn hostname_no_scheme() {
        let source = CitationSource::new().url("example.com/path");
        assert_eq!(source.hostname(), Some("example.com"));
    }

    #[test]
    fn hostname_url_with_query_no_path() {
        let source = CitationSource::new().url("https://example.com?foo=bar");
        assert_eq!(source.hostname(), Some("example.com"));
    }

    #[test]
    fn hostname_url_with_fragment() {
        let source = CitationSource::new().url("https://example.com#section");
        assert_eq!(source.hostname(), Some("example.com"));
    }

    #[test]
    fn hostname_strips_userinfo() {
        let source = CitationSource::new().url("https://user:pass@host.com/path");
        assert_eq!(source.hostname(), Some("host.com"));
    }

    #[test]
    fn hostname_empty_url() {
        let source = CitationSource::new().url("");
        assert_eq!(source.hostname(), None);
    }

    #[test]
    fn citation_source_builder() {
        let source = CitationSource::new()
            .title("Test")
            .url("https://test.com")
            .description("A desc")
            .snippet("snippet")
            .quote("a quote");
        assert_eq!(source.title.as_ref().map(|s| s.as_ref()), Some("Test"));
        assert_eq!(
            source.url.as_ref().map(|s| s.as_ref()),
            Some("https://test.com")
        );
        assert_eq!(
            source.description.as_ref().map(|s| s.as_ref()),
            Some("A desc")
        );
        assert_eq!(source.snippet.as_ref().map(|s| s.as_ref()), Some("snippet"));
        assert_eq!(source.quote.as_ref().map(|s| s.as_ref()), Some("a quote"));
    }
}
