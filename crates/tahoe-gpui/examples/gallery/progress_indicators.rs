//! Progress Indicators demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::status::activity_indicator::ActivityIndicator;
use tahoe_gpui::components::status::activity_ring::ActivityRing;
use tahoe_gpui::components::status::gauge::Gauge;
use tahoe_gpui::components::status::progress_indicator::ProgressIndicator;
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
        .id("progress-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_lg)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Progress Indicators"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "Progress indicators show that something is happening. Use \
                     determinate forms when you can estimate completion, indeterminate \
                     forms otherwise.",
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style_emphasized(TextStyle::Headline, theme)
                        .text_color(theme.text)
                        .child("Activity indicator (indeterminate)"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(theme.spacing_lg)
                        .child(ActivityIndicator::new("ai-1"))
                        .child(ActivityIndicator::new("ai-2").label("Loading\u{2026}")),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style_emphasized(TextStyle::Headline, theme)
                        .text_color(theme.text)
                        .child("Progress bar (determinate)"),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(theme.spacing_sm)
                        .child(div().w(px(360.0)).child(ProgressIndicator::new(0.0)))
                        .child(div().w(px(360.0)).child(ProgressIndicator::new(0.25)))
                        .child(div().w(px(360.0)).child(ProgressIndicator::new(0.50)))
                        .child(div().w(px(360.0)).child(ProgressIndicator::new(0.75)))
                        .child(div().w(px(360.0)).child(ProgressIndicator::new(1.0))),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style_emphasized(TextStyle::Headline, theme)
                        .text_color(theme.text)
                        .child("Gauge"),
                )
                .child(
                    div()
                        .flex()
                        .gap(theme.spacing_md)
                        .child(div().w(px(160.0)).child(Gauge::new(0.20)))
                        .child(div().w(px(160.0)).child(Gauge::new(0.55)))
                        .child(div().w(px(160.0)).child(Gauge::new(0.85))),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style_emphasized(TextStyle::Headline, theme)
                        .text_color(theme.text)
                        .child("Activity ring (Apple Fitness style)"),
                )
                .child(
                    div()
                        .flex()
                        .gap(theme.spacing_md)
                        .items_center()
                        .child(ActivityRing::new(0.30).size(px(48.0)))
                        .child(ActivityRing::new(0.60).size(px(48.0)))
                        .child(ActivityRing::new(0.90).size(px(48.0)))
                        .child(ActivityRing::new(1.0).size(px(48.0))),
                ),
        )
        .into_any_element()
}
