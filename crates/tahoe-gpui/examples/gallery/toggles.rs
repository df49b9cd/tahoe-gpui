//! Toggles (Toggle) demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::selection_and_input::toggle::Toggle;
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
    let toggle_on = state.toggle_on;

    div()
        .id("toggles-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Toggles"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A toggle has two states: on and off. People interact with a toggle \
                     to switch between these states.",
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
                        .w(px(180.0))
                        .child(if toggle_on {
                            "Interactive (ON)"
                        } else {
                            "Interactive (OFF)"
                        }),
                )
                .child({
                    let entity = entity.clone();
                    Toggle::new("sw-interactive").checked(toggle_on).on_change(
                        move |new_val, _window, cx| {
                            entity.update(cx, |this, cx| {
                                this.toggle_on = new_val;
                                cx.notify();
                            });
                        },
                    )
                }),
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
                        .w(px(180.0))
                        .child("Disabled off"),
                )
                .child(Toggle::new("sw-disabled-off").checked(false).disabled(true)),
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
                        .w(px(180.0))
                        .child("Disabled on"),
                )
                .child(Toggle::new("sw-disabled-on").checked(true).disabled(true)),
        )
        .into_any_element()
}
