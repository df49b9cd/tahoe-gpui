//! Scroll views demo (issue #156 F-09 + F-10). Exercises the
//! `ScrollView` container and the `scroll_edge_top` / `scroll_edge_bottom`
//! Liquid Glass overlays added in the July 28, 2025 HIG update.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::presentation::scroll_view::{ScrollAxis, ScrollView};
use tahoe_gpui::foundations::materials::{
    Elevation, Glass, SCROLL_EDGE_HEIGHT, ScrollEdgeStyle, Shape, glass_effect, scroll_edge_bottom,
    scroll_edge_top,
};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let card = |label: &'static str| {
        div()
            .h(px(64.0))
            .min_w(px(120.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(theme.radius_md)
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border)
            .text_style(TextStyle::Body, theme)
            .text_color(theme.text)
            .child(label)
    };

    let labels = [
        "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten", "Eleven",
        "Twelve",
    ];

    let edge_demo = |style: ScrollEdgeStyle, name: &'static str| {
        let inner = div()
            .id(format!("scroll-edge-inner-{}", name.to_lowercase()))
            .flex()
            .flex_col()
            .gap(theme.spacing_sm)
            .p(theme.spacing_md)
            .overflow_y_scroll()
            .size_full()
            .children(labels.iter().map(|l| {
                div()
                    .h(px(36.0))
                    .px(theme.spacing_sm)
                    .flex()
                    .items_center()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text)
                    .child(*l)
            }));

        div()
            .flex()
            .flex_col()
            .gap(theme.spacing_xs)
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text_muted)
                    .child(name),
            )
            .child(
                glass_effect(
                    div().w(px(220.0)).h(px(220.0)).relative().overflow_hidden(),
                    theme,
                    Glass::Regular,
                    Shape::Default,
                    Elevation::Elevated,
                )
                .child(inner)
                .child(scroll_edge_top(theme, SCROLL_EDGE_HEIGHT, style))
                .child(scroll_edge_bottom(theme, SCROLL_EDGE_HEIGHT, style)),
            )
    };

    div()
        .id("scroll-views-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Scroll Views"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "ScrollView wraps children in an axis-scrollable container. \
                     macOS 26 (July 28, 2025 HIG) introduced scroll edge effects: \
                     soft and hard fade overlays painted at the top/bottom edges \
                     so floating Liquid Glass toolbars stay legible over content.",
                ),
        )
        .child(
            div()
                .pt(theme.spacing_sm)
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Horizontal scroll"),
        )
        .child(
            div().h(px(96.0)).child(
                ScrollView::new("sv-horizontal")
                    .axis(ScrollAxis::Horizontal)
                    .gap(theme.spacing_sm)
                    .children(labels.iter().map(|l| card(l))),
            ),
        )
        .child(
            div()
                .pt(theme.spacing_md)
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Scroll edge effects"),
        )
        .child(
            div()
                .text_style(TextStyle::Subheadline, theme)
                .text_color(theme.text_muted)
                .child(
                    "Soft fades the content out gradually; Hard keeps content \
                     legible up to the edge then drops abruptly. Both use a \
                     colour-fade gradient today; a future upgrade will use \
                     `paint_blur_rect` with a variable-radius mask for the \
                     HIG-correct scroll-edge blur.",
                ),
        )
        .child(
            div()
                .pt(theme.spacing_sm)
                .flex()
                .gap(theme.spacing_lg)
                .child(edge_demo(ScrollEdgeStyle::Soft, "Soft edge"))
                .child(edge_demo(ScrollEdgeStyle::Hard, "Hard edge")),
        )
        .into_any_element()
}
