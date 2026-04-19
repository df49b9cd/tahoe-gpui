//! Gauge demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::components::status::gauge::{Gauge, GaugeStyle};
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
        .id("gauges-pane")
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
                        .child("Gauges"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "A gauge displays a value within a range, \
                             available in linear and circular styles.",
                        ),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Linear gauges"),
                )
                .child(Gauge::new(0.2).label("Low").style(GaugeStyle::Linear))
                .child(Gauge::new(0.5).label("Medium").style(GaugeStyle::Linear))
                .child(Gauge::new(0.9).label("High").style(GaugeStyle::Linear))
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Compact gauges"),
                )
                .child(
                    div()
                        .flex()
                        .gap(theme.spacing_xl)
                        .child(Gauge::new(0.3).label("30%").style(GaugeStyle::Compact))
                        .child(Gauge::new(0.65).label("65%").style(GaugeStyle::Compact))
                        .child(Gauge::new(1.0).label("100%").style(GaugeStyle::Compact)),
                ),
        )
        .into_any_element()
}
