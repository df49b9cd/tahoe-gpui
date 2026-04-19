//! ActivityRing demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::status::activity_ring::ActivityRing;
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
        .id("activity-rings-pane")
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
                        .child("Activity Rings"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "An activity ring shows a progress value on a \
                             circular track, inspired by Apple Watch rings.",
                        ),
                )
                .child(
                    div()
                        .flex()
                        .gap(theme.spacing_xl)
                        .items_center()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(theme.spacing_sm)
                                .child(ActivityRing::new(0.25).size(px(60.0)))
                                .child(
                                    div()
                                        .text_style(TextStyle::Caption1, theme)
                                        .text_color(theme.text_muted)
                                        .child("25%"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(theme.spacing_sm)
                                .child(ActivityRing::new(0.5).size(px(60.0)))
                                .child(
                                    div()
                                        .text_style(TextStyle::Caption1, theme)
                                        .text_color(theme.text_muted)
                                        .child("50%"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(theme.spacing_sm)
                                .child(ActivityRing::new(0.75).size(px(60.0)))
                                .child(
                                    div()
                                        .text_style(TextStyle::Caption1, theme)
                                        .text_color(theme.text_muted)
                                        .child("75%"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(theme.spacing_sm)
                                .child(ActivityRing::new(1.0).size(px(60.0)))
                                .child(
                                    div()
                                        .text_style(TextStyle::Caption1, theme)
                                        .text_color(theme.text_muted)
                                        .child("100%"),
                                ),
                        ),
                ),
        )
        .into_any_element()
}
