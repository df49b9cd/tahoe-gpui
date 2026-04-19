//! Steppers demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::selection_and_input::stepper::Stepper;
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
    let stepper_value = state.stepper_value;

    div()
        .id("steppers-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Steppers"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A stepper is a two-segment control that lets people increase \
                     or decrease an incremental value.",
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
                        .w(px(200.0))
                        .child(format!("Interactive ({stepper_value})")),
                )
                .child(
                    Stepper::new("st-interactive")
                        .value(stepper_value)
                        .min(0.0)
                        .max(100.0)
                        .on_change(move |new_val, _window, cx| {
                            entity.update(cx, |this, cx| {
                                this.stepper_value = new_val;
                                cx.notify();
                            });
                        }),
                ),
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
                        .w(px(200.0))
                        .child("Step 0.25"),
                )
                .child(
                    Stepper::new("st-step")
                        .value(2.5)
                        .min(0.0)
                        .max(10.0)
                        .step(0.25),
                ),
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
                        .w(px(200.0))
                        .child("At max"),
                )
                .child(Stepper::new("st-max").value(10.0).min(0.0).max(10.0)),
        )
        .into_any_element()
}
