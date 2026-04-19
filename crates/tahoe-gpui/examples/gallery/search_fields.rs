//! Search Fields demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::navigation_and_search::search_bar::SearchBar;
use tahoe_gpui::components::navigation_and_search::search_field::SearchField;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let entity = cx.entity().clone();
    let search_value = state.search_value.clone();
    let search_focus = state.search_focus.clone();
    let is_focused = state.search_focus.is_focused(window);

    let status_text = if search_value.is_empty() {
        "No search query".to_string()
    } else {
        format!("Searching: {search_value}")
    };

    div()
        .id("search-fields-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Search Fields"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child("A search field is a text input optimized for entering search queries."),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text)
                .child(status_text),
        )
        .child(
            div()
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("SearchField (interactive)"),
        )
        .child(
            div().w(px(360.0)).child(
                SearchField::new("sf-interactive")
                    .placeholder("Search documents")
                    .value(search_value)
                    .focus_handle(search_focus)
                    .focused(is_focused)
                    .on_change(move |new_val, _window, cx| {
                        entity.update(cx, |this, cx| {
                            this.search_value = new_val;
                            cx.notify();
                        });
                    }),
            ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("SearchBar (display-only capsule)"),
        )
        .child(
            div()
                .w(px(360.0))
                .child(SearchBar::new("sb-1").placeholder("Search\u{2026}")),
        )
        .into_any_element()
}
