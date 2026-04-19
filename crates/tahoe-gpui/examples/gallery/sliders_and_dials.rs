//! Sliders and Dials demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::status::gauge::{Gauge, GaugeStyle};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let slider_a = state.slider_a.clone();
    let slider_b = state.slider_b.clone();

    div()
        .id("sliders-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Sliders and Dials"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A slider lets people choose a value from a range. Drag the knob, \
                     click anywhere on the track, or press arrow keys to adjust.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(140.0))
                        .child("Slider (default)"),
                )
                .child(div().w(px(360.0)).child(slider_a)),
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
                        .w(px(140.0))
                        .child("Slider (mid)"),
                )
                .child(div().w(px(360.0)).child(slider_b)),
        )
        .child(div().h(px(16.0)))
        // ── Gauges / Dials section ──────────────────────────────────────
        .child(
            div()
                .text_style_emphasized(TextStyle::Title2, theme)
                .text_color(theme.text)
                .child("Gauges"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child("A gauge shows a value within a range using color-coded levels."),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(140.0))
                        .child("Low (green)"),
                )
                .child(div().w(px(360.0)).child(Gauge::new(0.2).label("20%"))),
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
                        .w(px(140.0))
                        .child("Medium (yellow)"),
                )
                .child(div().w(px(360.0)).child(Gauge::new(0.5).label("50%"))),
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
                        .w(px(140.0))
                        .child("High (red)"),
                )
                .child(div().w(px(360.0)).child(Gauge::new(0.85).label("85%"))),
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
                        .w(px(140.0))
                        .child("Compact style"),
                )
                .child(
                    div()
                        .w(px(360.0))
                        .child(Gauge::new(0.65).style(GaugeStyle::Compact)),
                ),
        )
        .into_any_element()
}
