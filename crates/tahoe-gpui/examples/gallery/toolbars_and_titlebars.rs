//! Toolbars and Titlebars demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, SharedString, Window, div, px};

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::navigation_and_search::toolbar::Toolbar;
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

    let toolbar = Toolbar::new("demo-toolbar")
        .leading(
            Button::new("toggle-sidebar")
                .icon(Icon::new(IconName::DevSidebar).size(px(14.0)))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm),
        )
        .leading(
            div()
                .flex()
                .gap(px(2.0))
                .child(
                    Button::new("nav-back")
                        .icon(Icon::new(IconName::ChevronLeft).size(px(14.0)))
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm),
                )
                .child(
                    Button::new("nav-forward")
                        .icon(Icon::new(IconName::ChevronRight).size(px(14.0)))
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm),
                ),
        )
        .title(SharedString::from("Documents"))
        .trailing(
            Button::new("act-new")
                .icon(Icon::new(IconName::FolderOpen).size(px(14.0)))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm),
        )
        .trailing(
            Button::new("act-trash")
                .icon(Icon::new(IconName::Trash).size(px(14.0)))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm),
        )
        .trailing(
            Button::new("act-search")
                .icon(Icon::new(IconName::Search).size(px(14.0)))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm),
        );

    div()
        .id("toolbars-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Toolbars and Titlebars"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A toolbar is a container for buttons and other controls in the \
                     window's titlebar area.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .border_1()
                .border_color(theme.border)
                .rounded(theme.radius_lg)
                .overflow_hidden()
                .child(toolbar),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .pt(theme.spacing_md)
                .child(
                    "See `toolbar_app.rs` and `window_layouts.rs` examples for the \
                     full window-chrome variants (Titlebar, Toolbar, Monobar, Toolbar \
                     No Nav, Utility Panel).",
                ),
        )
        .into_any_element()
}
