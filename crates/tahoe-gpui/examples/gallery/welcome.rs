//! Welcome page for the primitive gallery — explains how to use it.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

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
        .id("welcome-scroll")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("tahoe-gpui primitive gallery"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "Browse every primitive component side-by-side with its macOS 26 \
                     (Tahoe) Figma reference. Pick a primitive from the sidebar to see \
                     the demo in this pane.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style_emphasized(TextStyle::Title3, theme)
                .text_color(theme.text)
                .child("How this gallery is laid out"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "Each demo is a focused mirror of one page in the macOS 26 \
                     (Community) UI Kit by Apple Design Resources. Click any sidebar \
                     entry to switch to that primitive.",
                ),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "Inside each demo you can interact with the primitive — press, \
                     click, drag, type — to verify the behavior matches the canonical \
                     Apple component.",
                ),
        )
        .into_any_element()
}
