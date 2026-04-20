//! ButtonGroup demo (issue #156 F-09). Renders the segmented action-pair
//! pattern from HIG Menus & Actions.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::menus_and_actions::button_group::ButtonGroup;
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
        .id("button-groups-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Button Groups"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A button group renders related actions as a cohesive segmented \
                     unit, with a shared glass surface and 1pt hairline dividers \
                     between adjacent items.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Icon-only group"),
        )
        .child(
            ButtonGroup::new("bg-icons")
                .child(
                    Button::new("bg-play")
                        .icon(Icon::new(IconName::Sparkle).size(px(16.0)))
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Icon)
                        .tooltip("Sparkle"),
                )
                .child(
                    Button::new("bg-pause")
                        .icon(Icon::new(IconName::Bookmark).size(px(16.0)))
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Icon)
                        .tooltip("Bookmark"),
                )
                .child(
                    Button::new("bg-stop")
                        .icon(Icon::new(IconName::X).size(px(16.0)))
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Icon)
                        .tooltip("Close"),
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Labelled group"),
        )
        .child(
            ButtonGroup::new("bg-labels")
                .child(
                    Button::new("bg-day")
                        .label("Day")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Regular),
                )
                .child(
                    Button::new("bg-week")
                        .label("Week")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Regular),
                )
                .child(
                    Button::new("bg-month")
                        .label("Month")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Regular),
                ),
        )
        .into_any_element()
}
