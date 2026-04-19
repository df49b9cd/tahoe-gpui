//! Copy Buttons demo for the primitive gallery.
//!
//! CopyButton is stateful (Entity-based), so we show it via description
//! and disabled button placeholders. A full interactive demo would
//! require Entity state on ComponentGallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::foundations::icons::{Icon, IconName};
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
        .id("copy-buttons-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Copy Buttons"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A copy button writes text to the clipboard and shows a \
                     checkmark feedback state. It is stateful (Entity-based).",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Icon states"),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_lg)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(theme.spacing_xs)
                        .child(
                            Button::new("copy-idle")
                                .icon(Icon::new(IconName::Copy).size(px(16.0)))
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::IconSm),
                        )
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .child("Idle"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(theme.spacing_xs)
                        .child(
                            Button::new("copy-copied")
                                .icon(
                                    Icon::new(IconName::Check)
                                        .size(px(16.0))
                                        .color(theme.success),
                                )
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::IconSm),
                        )
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .child("Copied"),
                        ),
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .child(
                    "Create with CopyButton::new(\"text to copy\", cx). \
                     Supports set_timeout(), set_on_copy(), set_custom_child(), and set_disabled().",
                ),
        )
        .into_any_element()
}
