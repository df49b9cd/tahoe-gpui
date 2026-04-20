//! Sidebars demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::navigation_and_search::sidebar::{Sidebar, SidebarItem};
use tahoe_gpui::foundations::icons::IconName;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

const ITEMS: &[(IconName, &str)] = &[
    (IconName::Folder, "Inbox"),
    (IconName::Search, "Drafts"),
    (IconName::Send, "Sent"),
    (IconName::Trash, "Trash"),
    (IconName::Bookmark, "Starred"),
];

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let mut sidebar_content = div().flex().flex_col().size_full().pt(theme.spacing_md);
    for (i, (icon, label)) in ITEMS.iter().enumerate() {
        sidebar_content = sidebar_content.child(
            SidebarItem::new(format!("demo-sidebar-item-{i}"), *label)
                .icon(*icon)
                .selected(i == 0),
        );
    }

    let sidebar_demo = Sidebar::new("demo-sidebar")
        .width(px(220.0))
        .child(sidebar_content);

    div()
        .id("sidebars-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Sidebars"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A sidebar provides app-level navigation and organizes content \
                     into a hierarchy. Rows use the `SidebarItem` primitive, which \
                     provides keyboard focus, Enter/Space activation, a visible \
                     focus ring, and a platform-appropriate minimum row height \
                     (28 pt macOS, 44 pt iOS/iPadOS/watchOS).",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .h(px(360.0))
                .rounded(theme.radius_md)
                .overflow_hidden()
                .flex()
                .bg(theme.surface)
                .child(sidebar_demo)
                .child(
                    div()
                        .flex_1()
                        .p(theme.spacing_md)
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child("Main content area"),
                ),
        )
        .into_any_element()
}
