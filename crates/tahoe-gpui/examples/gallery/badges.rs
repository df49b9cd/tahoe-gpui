//! Badges demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::content::badge::{Badge, BadgeVariant};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let row = |label: &'static str, variant: BadgeVariant| {
        div()
            .flex()
            .items_center()
            .gap(theme.spacing_md)
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text)
                    .w(px(120.0))
                    .child(label),
            )
            .child(Badge::new(label).variant(variant))
    };

    div()
        .id("badges-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Badges"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A badge is a small status pill conveying semantic meaning \
                     through color and label text.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(row("Default", BadgeVariant::Default))
        .child(row("Success", BadgeVariant::Success))
        .child(row("Warning", BadgeVariant::Warning))
        .child(row("Error", BadgeVariant::Error))
        .child(row("Info", BadgeVariant::Info))
        .child(row("Muted", BadgeVariant::Muted))
        .into_any_element()
}
