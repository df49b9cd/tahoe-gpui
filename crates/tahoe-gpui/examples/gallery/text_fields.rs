//! Text Fields demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let empty = state.text_input_empty.clone();
    let filled = state.text_input_filled.clone();

    div()
        .id("text-fields-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Text Fields"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A text field accepts a single line of user input. Click to focus, \
                     type to enter text, drag to select.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(140.0))
                        .child("Empty (placeholder)"),
                )
                .child(div().w(px(280.0)).child(empty)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(140.0))
                        .child("Pre-filled"),
                )
                .child(div().w(px(280.0)).child(filled)),
        )
        .into_any_element()
}
