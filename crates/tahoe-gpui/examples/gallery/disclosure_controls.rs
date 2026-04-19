//! Disclosure Controls demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::layout_and_organization::disclosure::Disclosure;
use tahoe_gpui::components::layout_and_organization::disclosure_group::DisclosureGroup;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let entity = cx.entity().clone();
    let disclosure_open = state.disclosure_open;

    div()
        .id("disclosure-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Disclosure Controls"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child("A disclosure control reveals or hides additional information."),
        )
        .child(div().h(px(12.0)))
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
                        .child("Standalone arrows"),
                )
                .child(Disclosure::new("d-collapsed").expanded(false))
                .child(Disclosure::new("d-expanded").expanded(true)),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text)
                .child("Interactive DisclosureGroup (click to toggle):"),
        )
        .child(
            DisclosureGroup::new(
                "c-1",
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text)
                    .child("Advanced settings"),
                div()
                    .p(theme.spacing_md)
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text_muted)
                    .child(
                        "When this group is expanded, additional options become \
                         visible. The header chevron rotates between right (collapsed) \
                         and down (expanded).",
                    ),
            )
            .open(disclosure_open)
            .on_toggle(move |new_open, _window, cx| {
                entity.update(cx, |this, cx| {
                    this.disclosure_open = new_open;
                    cx.notify();
                });
            }),
        )
        .into_any_element()
}
