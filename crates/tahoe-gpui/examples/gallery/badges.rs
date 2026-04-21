//! Badges demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::content::badge::{Badge, BadgeVariant};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let row = |label: &'static str, variant: BadgeVariant| {
        div()
            .flex()
            .items_center()
            .gap(theme.spacing_md)
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text)
                    .w(px(120.0))
                    .child(label),
            )
            .child(Badge::new(label).variant(variant))
    };

    let section_header = |title: &'static str| {
        div()
            .text_style(TextStyle::Headline, theme)
            .text_color(theme.text)
            .child(title)
    };

    div()
        .id("badges-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Badges"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A badge is a small status pill conveying semantic meaning \
                     through color and label text.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        // Semantic pill variants
        .child(section_header("Semantic Pills"))
        .child(row("Default", BadgeVariant::Default))
        .child(row("Success", BadgeVariant::Success))
        .child(row("Warning", BadgeVariant::Warning))
        .child(row("Error", BadgeVariant::Error))
        .child(row("Info", BadgeVariant::Info))
        .child(row("Muted", BadgeVariant::Muted))
        .child(div().h(theme.spacing_sm))
        // Notification badges
        .child(section_header("Notification Badges"))
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child("Opaque red pills with unread counts."),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(120.0))
                        .child("0"),
                )
                .child(Badge::notification(0)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(120.0))
                        .child("5"),
                )
                .child(Badge::notification(5)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(120.0))
                        .child("99"),
                )
                .child(Badge::notification(99)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(120.0))
                        .child("100 (capped)"),
                )
                .child(Badge::notification(100)),
        )
        .child(div().h(theme.spacing_sm))
        // Dot variant
        .child(section_header("Dot Variant"))
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child("8 pt solid circle for silent presence / unread indicators."),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(120.0))
                        .child("Dot"),
                )
                .child(Badge::dot()),
        )
        .child(div().h(theme.spacing_sm))
        // Interactive badges
        .child(section_header("Interactive Badges"))
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child("Filter chips with HIG-compliant 20 pt minimum height."),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(120.0))
                        .child("Filter"),
                )
                .child(Badge::new("Filter").interactive(true)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(120.0))
                        .child("Active"),
                )
                .child(
                    Badge::new("Active")
                        .variant(BadgeVariant::Success)
                        .interactive(true),
                ),
        )
        .into_any_element()
}
