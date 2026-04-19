//! SplitView demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let split_view = state.split_view.clone();

    div()
        .id("split-views-pane")
        .child(
            div()
                .p(theme.spacing_xl)
                .flex()
                .flex_col()
                .gap(theme.spacing_lg)
                .child(
                    div()
                        .text_style_emphasized(TextStyle::LargeTitle, theme)
                        .text_color(theme.text)
                        .child("Split Views"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "A split view manages two resizable panes with a \
                             draggable divider. Drag the divider or use arrow keys.",
                        ),
                )
                .child(
                    div()
                        .h(px(300.0))
                        .w_full()
                        .border_1()
                        .border_color(theme.border)
                        .rounded(theme.radius_md)
                        .overflow_hidden()
                        .child(split_view),
                ),
        )
        .into_any_element()
}
