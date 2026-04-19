//! Token Fields demo. Wires a live `Entity<TokenField>` from the gallery
//! state — Enter/comma commits a new token, the X glyph removes one, and
//! suggestions appear under the chip row when the field is focused.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let token_field = state.token_field.clone();

    div()
        .id("token-fields-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Token Fields"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A token field manages a list of tag/chip tokens with inline \
                     text entry. Enter or comma adds a token, Backspace on the empty \
                     input removes the last one, and the X glyph removes a specific \
                     chip. Try typing a new tag below.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Live token field"),
        )
        .child(token_field)
        .into_any_element()
}
