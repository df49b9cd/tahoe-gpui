//! Notifications demo. macOS notifications are normally posted by the OS via
//! `UNUserNotificationCenter`; this demo shows the visual appearance of a
//! notification banner composed from existing primitives.

use gpui::prelude::*;
use gpui::{AnyElement, Context, FontWeight, Window, div, px};

use tahoe_gpui::components::content::avatar::Avatar;
use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::foundations::materials::{Elevation, Glass, Shape, glass_effect};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let banner = glass_effect(
        div().w(px(360.0)).overflow_hidden(),
        theme,
        Glass::Regular,
        Shape::Default,
        Elevation::Elevated,
    )
    .flex()
    .flex_col();

    let banner = banner
        .child(
            div()
                .flex()
                .items_start()
                .gap(theme.spacing_sm)
                .p(theme.spacing_md)
                // Use the theme accent so the demo avatar follows
                // light/dark mode and brand color changes instead of being
                // pinned to a hardcoded blue.
                .child(Avatar::new("M").bg(theme.accent).size(px(36.0)))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_style(TextStyle::Subheadline, theme)
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.text)
                                        .child("Messages"),
                                )
                                .child(
                                    div()
                                        .text_style(TextStyle::Caption2, theme)
                                        .text_color(theme.text_muted)
                                        .child("now"),
                                ),
                        )
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text)
                                .child("Søren"),
                        )
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text)
                                .child("Hey, are you free for lunch tomorrow?"),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .gap(theme.spacing_xs)
                .p(theme.spacing_sm)
                .child(
                    Button::new("notif-reply")
                        .label("Reply")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Small),
                )
                .child(
                    Button::new("notif-dismiss")
                        .label("Dismiss")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Small),
                ),
        );

    div()
        .id("notifications-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Notifications"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "On macOS, notifications are posted by the system via \
                     UNUserNotificationCenter. Here\u{2019}s the visual treatment of a \
                     notification banner composed from tahoe-gpui primitives.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(banner)
        .into_any_element()
}
