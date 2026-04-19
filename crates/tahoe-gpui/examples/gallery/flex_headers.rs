//! FlexHeader demo (issue #156 F-09). FlexHeader is the standard
//! horizontal header used by the code display components (terminal,
//! commit, stack-trace, artifact). The gallery shows the four toggle
//! permutations: padding, gap, border, alignment.

use gpui::prelude::*;
use gpui::{AnyElement, Context, FontWeight, Window, div};

use tahoe_gpui::components::layout_and_organization::flex_header::{FlexAlign, FlexHeader};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let title = |label: &'static str| {
        div()
            .text_style(TextStyle::Body, theme)
            .text_color(theme.text)
            .font_weight(FontWeight::MEDIUM)
            .child(label)
    };

    let subtitle = |label: &'static str| {
        div()
            .text_style(TextStyle::Caption1, theme)
            .text_color(theme.text_muted)
            .child(label)
    };

    div()
        .id("flex-headers-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Flex Headers"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "FlexHeader is a configurable horizontal header for code \
                     display components. Toggle padding, gap, border, and \
                     alignment to match the surrounding chrome.",
                ),
        )
        .child(subtitle("Default (padded, justify_between, items_center)"))
        .child(
            FlexHeader::new()
                .child(title("terminal.rs"))
                .child(subtitle("Modified")),
        )
        .child(subtitle("With border + gap"))
        .child(
            FlexHeader::new()
                .border(true)
                .gap(true)
                .child(title("commit.rs"))
                .child(subtitle("Staged")),
        )
        .child(subtitle("Items start (multi-line right column)"))
        .child(
            FlexHeader::new()
                .border(true)
                .align(FlexAlign::Start)
                .child(title("stack_trace.rs"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(subtitle("Frame 0"))
                        .child(subtitle("frame::call")),
                ),
        )
        .into_any_element()
}
