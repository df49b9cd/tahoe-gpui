//! Image Wells demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::components::selection_and_input::image_well::ImageWell;
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
        .id("image-wells-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Image Wells"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child("An image well displays an editable image preview."),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .flex()
                .gap(theme.spacing_lg)
                .items_center()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(theme.spacing_xs)
                        .child(ImageWell::new("iw-empty"))
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .child("Empty"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(theme.spacing_xs)
                        .child(
                            ImageWell::new("iw-filled")
                                .image_url("https://example.invalid/avatar.png"),
                        )
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .child("With URL"),
                        ),
                ),
        )
        .into_any_element()
}
