//! Pointers demo. Pointers are not a discrete primitive — they're a system
//! concern (cursor styles set per-element via GPUI's `cursor_*` Styled APIs).
//! This demo lists the available cursor styles with hoverable swatches.

use gpui::prelude::*;
use gpui::{AnyElement, Context, CursorStyle, Window, div, px};

use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

const STYLES: &[(&str, CursorStyle)] = &[
    ("Arrow", CursorStyle::Arrow),
    ("IBeam (text)", CursorStyle::IBeam),
    ("Pointing hand", CursorStyle::PointingHand),
    ("Closed hand", CursorStyle::ClosedHand),
    ("Open hand", CursorStyle::OpenHand),
    ("Crosshair", CursorStyle::Crosshair),
    ("Resize left", CursorStyle::ResizeLeft),
    ("Resize right", CursorStyle::ResizeRight),
    ("Resize up", CursorStyle::ResizeUp),
    ("Resize down", CursorStyle::ResizeDown),
    ("Resize column", CursorStyle::ResizeColumn),
    ("Resize row", CursorStyle::ResizeRow),
    ("Operation not allowed", CursorStyle::OperationNotAllowed),
    ("Drag link", CursorStyle::DragLink),
    ("Drag copy", CursorStyle::DragCopy),
    ("Contextual menu", CursorStyle::ContextualMenu),
];

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let mut grid = div().flex().flex_wrap().gap(theme.spacing_md);
    for (label, cursor) in STYLES {
        grid = grid.child(
            div()
                .id(*label)
                .w(px(180.0))
                .h(px(48.0))
                .px(theme.spacing_md)
                .flex()
                .items_center()
                .border_1()
                .border_color(theme.border)
                .rounded(theme.radius_md)
                .bg(theme.surface)
                .cursor(*cursor)
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text)
                .child(*label),
        );
    }

    div()
        .id("pointers-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Pointers"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "Pointer styles are set per-element via GPUI's cursor_* Styled \
                     APIs. Hover any swatch below to see the corresponding system \
                     cursor.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(grid)
        .into_any_element()
}
