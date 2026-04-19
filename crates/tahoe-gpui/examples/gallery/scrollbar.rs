//! Scrollbar demo. Scrollbars are a GPUI built-in (overlay scrollbars on
//! `overflow_y_scroll()` containers); this demo shows them in action.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    // Build a tall column of dummy rows so the scrollbar appears.
    let mut rows = div().flex().flex_col();
    for i in 1..=80 {
        rows = rows.child(
            div()
                .py(theme.spacing_sm)
                .px(theme.spacing_md)
                .border_b_1()
                .border_color(theme.border)
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text)
                .child(format!("Row {i}")),
        );
    }

    div()
        .id("scrollbar-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Scrollbar"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "Scrollbars are an overlay element provided by GPUI. They appear \
                     automatically on `overflow_y_scroll()` containers and follow \
                     the system appearance.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .id("scroll-container")
                .max_h(px(360.0))
                .border_1()
                .border_color(theme.border)
                .rounded(theme.radius_lg)
                .overflow_hidden()
                .child(
                    div()
                        .id("scroll-inner")
                        .h_full()
                        .overflow_y_scroll()
                        .child(rows),
                ),
        )
        .into_any_element()
}
