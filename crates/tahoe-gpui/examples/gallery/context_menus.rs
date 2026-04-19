//! Context Menus demo for the primitive gallery.
//!
//! Demonstrates a live ContextMenu entity that opens on right-click.

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
    let status_text = state.context_menu_status.clone();

    div()
        .id("context-menus-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Context Menus"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A context menu presents a list of actions related to the \
                     current context, typically triggered by right-click.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text)
                .child(status_text),
        )
        .child(
            div()
                .id("ctx-target-area")
                .min_h(px(120.0))
                .w_full()
                .flex()
                .items_center()
                .justify_center()
                .rounded(theme.radius_lg)
                .border_2()
                .border_color(theme.border)
                .bg(theme.surface)
                .cursor_pointer()
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child("Right-click here to open context menu"),
                )
                .on_mouse_down(MouseButton::Right, {
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
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .child(
                    "Supports: Default, Destructive, Disabled item styles; \
                     optional leading icons; keyboard shortcut hints.",
                ),
        )
        .into_any_element()
}
