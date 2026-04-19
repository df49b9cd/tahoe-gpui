//! Navigation Bars demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::navigation_and_search::navigation_bar::NavigationBarIOS as NavigationBar;
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
        .id("navigation-bars-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Navigation Bars"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A navigation bar appears at the top of a view with a centered \
                     title and optional leading/trailing action areas.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        // Title only
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Title only"),
        )
        .child(NavigationBar::new("nav-title").title("Settings"))
        .child(div().h(theme.spacing_sm))
        // With back button
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("With back button"),
        )
        .child(
            NavigationBar::new("nav-back").title("Details").leading(
                Button::new("back-btn")
                    .icon(Icon::new(IconName::ChevronLeft).size(px(16.0)))
                    .label("Back")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm),
            ),
        )
        .child(div().h(theme.spacing_sm))
        // With trailing action
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("With trailing action"),
        )
        .child(
            NavigationBar::new("nav-trailing").title("Inbox").trailing(
                Button::new("compose-btn")
                    .icon(Icon::new(IconName::Pencil).size(px(16.0)))
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm),
            ),
        )
        .into_any_element()
}
