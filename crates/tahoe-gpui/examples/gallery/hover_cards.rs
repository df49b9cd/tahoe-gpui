//! HoverCard demo for the primitive gallery.

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
    let hover_card = state.hover_card.clone();

    // Configure trigger and content each render (they read theme globals).
    hover_card.update(cx, |card, cx| {
        card.set_trigger(
            |cx| {
                let theme = cx.global::<TahoeTheme>();
                div()
                    .px(theme.spacing_md)
                    .py(theme.spacing_sm)
                    .rounded(theme.radius_md)
                    .bg(theme.surface)
                    .border_1()
                    .border_color(theme.border)
                    .cursor_pointer()
                    .child(
                        div()
                            .text_style(TextStyle::Body, theme)
                            .text_color(theme.accent)
                            .child("Hover me to see a card"),
                    )
                    .into_any_element()
            },
            cx,
        );

        card.set_content(
            |cx| {
                let theme = cx.global::<TahoeTheme>();
                div()
                    .p(theme.spacing_md)
                    .min_w(px(200.0))
                    .flex()
                    .flex_col()
                    .gap(theme.spacing_sm)
                    .child(
                        div()
                            .text_style(TextStyle::Headline, theme)
                            .text_color(theme.text)
                            .child("HoverCard Content"),
                    )
                    .child(
                        div()
                            .text_style(TextStyle::Body, theme)
                            .text_color(theme.text_muted)
                            .child(
                                "This card appears on hover and stays \
                             visible while hovering over the trigger \
                             or the card itself.",
                            ),
                    )
                    .into_any_element()
            },
            cx,
        );
    });

    div()
        .id("hover-cards-pane")
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
                        .child("Hover Cards"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "A hover card shows rich content when the user \
                             hovers over a trigger element. Unlike tooltips, \
                             hover cards can contain interactive content.",
                        ),
                )
                .child(div().h(theme.spacing_sm))
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .child("Hover over the label below:"),
                )
                .child(hover_card),
        )
        .into_any_element()
}
