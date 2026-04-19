//! Separator demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::layout_and_organization::separator::Separator;
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
        .id("separators-pane")
        .child(
            div()
                .p(theme.spacing_xl)
                .flex()
                .flex_col()
                .gap(theme.spacing_lg)
                .child(
                    div()
                        .text_style_emphasized(TextStyle::LargeTitle, theme)
                        .text_color(theme.text)
                        .child("Separators"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "A separator creates a visual division between \
                             content. Available in horizontal and vertical \
                             orientations.",
                        ),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Horizontal"),
                )
                .child(Separator::horizontal())
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child("Content between separators."),
                )
                .child(Separator::horizontal())
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Vertical (within a row)"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(theme.spacing_md)
                        .h(px(40.0))
                        .child(
                            div()
                                .text_style(TextStyle::Body, theme)
                                .text_color(theme.text)
                                .child("Left"),
                        )
                        .child(Separator::vertical())
                        .child(
                            div()
                                .text_style(TextStyle::Body, theme)
                                .text_color(theme.text)
                                .child("Center"),
                        )
                        .child(Separator::vertical())
                        .child(
                            div()
                                .text_style(TextStyle::Body, theme)
                                .text_color(theme.text)
                                .child("Right"),
                        ),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Custom color"),
                )
                .child(Separator::horizontal().color(theme.accent)),
        )
        .into_any_element()
}
