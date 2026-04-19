//! Menus demo — wires the live `ContextMenu` entity (issue #156 F-05).
//!
//! Reuses the gallery's shared `context_menu` Entity so the same menu
//! payload (Cut/Copy/Paste/Delete) is exercised here under a left-click
//! trigger and on the Context Menus page under a right-click trigger. The
//! live entity exposes the actual `ContextMenu.open(position, ...)` flow
//! that adopters need to wire.

use gpui::prelude::*;
use gpui::{AnyElement, Context, MouseButton, Window, div, px};

use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let context_menu = state.context_menu.clone();

    div()
        .id("menus-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Menus"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A menu reveals a set of choices people can make about an item, \
                     a process, or some other aspect of an app. Click the trigger \
                     below to open a live `ContextMenu` instance.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .id("menu-trigger")
                .min_h(px(44.0))
                .w(px(220.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded(theme.radius_md)
                .border_1()
                .border_color(theme.border)
                .bg(theme.surface)
                .cursor_pointer()
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .child("Click to open menu"),
                )
                .on_mouse_down(MouseButton::Left, {
                    let menu = context_menu.clone();
                    move |event, window, cx| {
                        menu.update(cx, |menu, cx| {
                            menu.open(event.position, window, cx);
                        });
                    }
                }),
        )
        .child(context_menu.clone())
        .child(
            div()
                .pt(theme.spacing_lg)
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .child(
                    "ContextMenu is entity-based; open at a screen position via \
                     `entity.update(cx, |menu, cx| menu.open(position, window, cx))`. \
                     The same entity also powers the Context Menus page under a \
                     right-click trigger.",
                ),
        )
        .into_any_element()
}
