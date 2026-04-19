//! Menu Bar and Dock demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::menus_and_actions::menu_bar::{Menu, MenuBar};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let menu_item = |label: &'static str| {
        div()
            .px(theme.spacing_sm)
            .py(px(4.0))
            .text_style(TextStyle::Body, theme)
            .text_color(theme.text)
            .child(label)
    };

    let menu_column = |labels: &'static [&'static str]| {
        let mut col = div().flex().flex_col();
        for label in labels {
            col = col.child(menu_item(label));
        }
        col
    };

    let menu_bar = MenuBar::new("demo-menu-bar").menus(vec![
        Menu::new(
            "File",
            menu_column(&[
                "New\u{2026}",
                "Open\u{2026}",
                "Save",
                "Save As\u{2026}",
                "Close Window",
            ]),
        ),
        Menu::new(
            "Edit",
            menu_column(&["Undo", "Redo", "Cut", "Copy", "Paste", "Select All"]),
        ),
        Menu::new(
            "View",
            menu_column(&["Show Sidebar", "Show Toolbar", "Enter Full Screen"]),
        ),
        Menu::new(
            "Window",
            menu_column(&[
                "Minimize",
                "Zoom",
                "Tile to Left of Screen",
                "Bring All to Front",
            ]),
        ),
        Menu::new(
            "Help",
            menu_column(&["Search", "Documentation", "Report a Bug"]),
        ),
    ]);

    div()
        .id("menu-bar-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Menu Bar and Dock"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "The menu bar groups related commands into a system of menus at \
                     the top of the screen.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .border_1()
                .border_color(theme.border)
                .rounded(theme.radius_lg)
                .overflow_hidden()
                .child(menu_bar),
        )
        .into_any_element()
}
