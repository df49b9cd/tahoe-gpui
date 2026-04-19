//! ActivityIndicator demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::status::activity_indicator::ActivityIndicator;
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
        .id("activity-indicators-pane")
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
                        .child("Activity Indicators"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "An activity indicator shows that a task is in \
                             progress. Use it for indeterminate loading states.",
                        ),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Default (spinning)"),
                )
                .child(ActivityIndicator::new("ai-default"))
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("With label"),
                )
                .child(ActivityIndicator::new("ai-label").label("Loading..."))
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Large"),
                )
                .child(
                    ActivityIndicator::new("ai-large")
                        .size(px(32.0))
                        .label("Please wait"),
                ),
        )
        .into_any_element()
}
