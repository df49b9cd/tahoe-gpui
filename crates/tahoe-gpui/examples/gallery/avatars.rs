//! Avatars demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::content::avatar::Avatar;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    div()
        .id("avatars-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Avatars"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "An avatar displays initials as a circular badge, \
                     typically representing a user or assistant.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        // Row: different sizes
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Sizes"),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(Avatar::new("U").size(px(24.0)))
                .child(Avatar::new("U").size(px(32.0)))
                .child(Avatar::new("U"))
                .child(Avatar::new("U").size(px(48.0)))
                .child(Avatar::new("U").size(px(64.0))),
        )
        // Row: different initials
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Initials"),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(Avatar::new("A"))
                .child(Avatar::new("SM"))
                .child(Avatar::new("JD")),
        )
        // Row: custom background colors
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Custom Colors"),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(Avatar::new("R").bg(theme.error))
                .child(Avatar::new("G").bg(theme.success))
                .child(Avatar::new("B").bg(theme.info)),
        )
        .into_any_element()
}
