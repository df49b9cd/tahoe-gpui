//! Popovers demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::presentation::popover::{Popover, PopoverPlacement};
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
    let popover_open = state.popover_open;

    let popover_content = |body: &'static str| {
        div()
            .w(px(240.0))
            .p(theme.spacing_md)
            .flex()
            .flex_col()
            .gap(theme.spacing_xs)
            .child(
                div()
                    .text_style_emphasized(TextStyle::Headline, theme)
                    .text_color(theme.text)
                    .child("Popover"),
            )
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text_muted)
                    .child(body),
            )
    };

    let entity_bl = entity.clone();
    let entity_br = entity.clone();
    let entity_dismiss_bl = entity.clone();
    let entity_dismiss_br = entity.clone();

    div()
        .id("popovers-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Popovers"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A popover presents related transient content directly relative \
                     to the element that triggered it. Click a button to toggle.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .flex()
                .gap(theme.spacing_lg)
                .child(
                    Popover::new(
                        "pop-below-left",
                        Button::new("pb-bl")
                            .label("Below Left")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Md)
                            .on_click({
                                let entity = entity_bl;
                                move |_, _, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.popover_open = if this.popover_open == Some(0) {
                                            None
                                        } else {
                                            Some(0)
                                        };
                                        cx.notify();
                                    });
                                }
                            }),
                        popover_content("This popover anchors below the trigger and aligns left."),
                    )
                    .placement(PopoverPlacement::BelowLeft)
                    .visible(popover_open == Some(0))
                    .on_dismiss(move |_window, cx| {
                        entity_dismiss_bl.update(cx, |this, cx| {
                            this.popover_open = None;
                            cx.notify();
                        });
                    }),
                )
                .child(
                    Popover::new(
                        "pop-below-right",
                        Button::new("pb-br")
                            .label("Below Right")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Md)
                            .on_click({
                                let entity = entity_br;
                                move |_, _, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.popover_open = if this.popover_open == Some(1) {
                                            None
                                        } else {
                                            Some(1)
                                        };
                                        cx.notify();
                                    });
                                }
                            }),
                        popover_content("This popover aligns right."),
                    )
                    .placement(PopoverPlacement::BelowRight)
                    .visible(popover_open == Some(1))
                    .on_dismiss(move |_window, cx| {
                        entity_dismiss_br.update(cx, |this, cx| {
                            this.popover_open = None;
                            cx.notify();
                        });
                    }),
                ),
        )
        .into_any_element()
}
