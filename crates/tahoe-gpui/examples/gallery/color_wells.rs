//! Color Wells demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::selection_and_input::color_well::ColorWell;
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
    let color = state.color_well_color;
    let color_open = state.color_well_open;

    div()
        .id("color-wells-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Color Wells"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A color well displays the current color of an element \
                     and lets people open a color picker to change it.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .flex()
                .gap(theme.spacing_md)
                .items_center()
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(140.0))
                        .child("Interactive"),
                )
                .child({
                    let entity_change = entity.clone();
                    let entity_toggle = entity.clone();
                    ColorWell::new("cw-interactive")
                        .color(color)
                        .open(color_open)
                        .on_change(move |new_color, _window, cx| {
                            entity_change.update(cx, |this, cx| {
                                this.color_well_color = new_color;
                                this.color_well_open = false;
                                cx.notify();
                            });
                        })
                        .on_toggle(move |open, _window, cx| {
                            entity_toggle.update(cx, |this, cx| {
                                this.color_well_open = open;
                                cx.notify();
                            });
                        })
                }),
        )
        .child(
            div()
                .flex()
                .gap(theme.spacing_md)
                .items_center()
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(140.0))
                        .child("Static samples"),
                )
                .child(ColorWell::new("cw-blue").color(theme.palette.blue))
                .child(ColorWell::new("cw-green").color(theme.palette.green))
                .child(ColorWell::new("cw-purple").color(theme.palette.purple)),
        )
        .into_any_element()
}
