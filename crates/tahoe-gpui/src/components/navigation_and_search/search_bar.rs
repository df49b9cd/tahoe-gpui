//! Search bar component with Apple glass material.
//!
//! Capsule-shaped search bar with a search icon and text display. By
//! default this is a **display-only** affordance used as the collapsed
//! state in toolbar search slots — click it to promote the interaction
//! into a full [`SearchField`](super::SearchField).
//!
//! HIG (iPadOS / macOS): "Put a search field at the trailing side of the
//! toolbar … the persistent availability of search at the side of the
//! toolbar gives it a global presence." That affordance must be
//! interactive. When a caller wires [`SearchBar::on_activate`], the bar
//! becomes clickable and fires the callback on click or Enter/Space —
//! typically the host then swaps in a focused `SearchField`.

use gpui::prelude::*;
use gpui::{App, ClickEvent, ElementId, KeyDownEvent, SharedString, Window, div, px};

use crate::callback_types::OnMutCallback;
use crate::foundations::icons::Icon;
use crate::foundations::icons::IconName;
use crate::foundations::materials::{LensEffect, SurfaceContext, glass_lens_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};

/// A capsule-shaped search affordance with glass material.
///
/// Renders a search icon followed by either the current value or
/// placeholder text. Wire [`SearchBar::on_activate`] to make it
/// interactive — without it, the bar is display-only and ignores clicks.
/// For an editable text field, use [`SearchField`](super::SearchField).
#[derive(IntoElement)]
pub struct SearchBar {
    id: ElementId,
    placeholder: SharedString,
    value: Option<SharedString>,
    on_activate: OnMutCallback,
}

impl SearchBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            placeholder: SharedString::from("Search"),
            value: None,
            on_activate: None,
        }
    }

    /// Set the placeholder text shown when the value is empty.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set the current search value.
    pub fn value(mut self, val: impl Into<SharedString>) -> Self {
        self.value = Some(val.into());
        self
    }

    /// Callback fired when the user clicks the bar or presses Enter /
    /// Space while it has keyboard focus. Callers typically use this to
    /// transition their UI into a focused [`SearchField`](super::SearchField)
    /// (matching the iPadOS / macOS "collapsed → expanded" search pattern).
    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SearchBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let display_text = match self.value.as_ref() {
            Some(v) if !v.is_empty() => v.clone(),
            _ => self.placeholder.clone(),
        };

        let has_value = self
            .value
            .as_deref()
            .map(|v| !v.is_empty())
            .unwrap_or(false);

        let text_color = if has_value {
            theme.label_color(SurfaceContext::GlassDim)
        } else {
            theme.secondary_label_color(SurfaceContext::GlassDim)
        };

        let icon_size = (TextStyle::Body.attrs().size * 0.9).ceil();
        let has_handler = self.on_activate.is_some();

        let mut effect = LensEffect::subtle(GlassSize::Small, theme);
        effect.blur.corner_radius = f32::from(theme.radius_full);
        let mut bar = glass_lens_surface(theme, &effect, GlassSize::Small)
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing_sm)
            .min_h(px(theme.target_size()))
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .rounded(theme.radius_full)
            .id(self.id)
            .debug_selector(|| "search-bar-root".into())
            .child(
                Icon::new(IconName::Search)
                    .size(icon_size)
                    .color(theme.secondary_label_color(SurfaceContext::GlassDim)),
            )
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(text_color)
                    .child(display_text),
            );

        if let Some(handler) = self.on_activate {
            let handler = std::rc::Rc::new(handler);
            let click_handler = handler.clone();
            let key_handler = handler;
            bar = bar
                .focusable()
                .cursor_pointer()
                .on_click(move |_event: &ClickEvent, window, cx| {
                    click_handler(window, cx);
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if crate::foundations::keyboard::is_activation_key(event) {
                        cx.stop_propagation();
                        key_handler(window, cx);
                    }
                });
        } else {
            let _ = has_handler;
        }

        bar
    }
}

#[cfg(test)]
mod tests {
    use super::SearchBar;
    use core::prelude::v1::test;

    #[test]
    fn search_bar_new_defaults() {
        let bar = SearchBar::new("search");
        assert_eq!(bar.placeholder.as_ref(), "Search");
        assert!(bar.value.is_none());
        assert!(bar.on_activate.is_none());
    }

    #[test]
    fn search_bar_builder_placeholder() {
        let bar = SearchBar::new("search").placeholder("Find...");
        assert_eq!(bar.placeholder.as_ref(), "Find...");
    }

    #[test]
    fn search_bar_builder_value() {
        let bar = SearchBar::new("search").value("hello");
        assert_eq!(bar.value.unwrap().as_ref(), "hello");
    }

    #[test]
    fn search_bar_builder_all_fields() {
        let bar = SearchBar::new("search")
            .placeholder("Type here")
            .value("query");
        assert_eq!(bar.placeholder.as_ref(), "Type here");
        assert_eq!(bar.value.unwrap().as_ref(), "query");
    }

    #[test]
    fn search_bar_default_placeholder() {
        let bar = SearchBar::new("search");
        assert_eq!(bar.placeholder.as_ref(), "Search");
    }

    #[test]
    fn search_bar_on_activate_builder() {
        let bar = SearchBar::new("search").on_activate(|_, _| {});
        assert!(bar.on_activate.is_some());
    }
}
