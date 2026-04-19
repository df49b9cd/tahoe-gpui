//! Windows demo (issue #156 F-09). The component gallery cannot open
//! sibling NSWindows from inside a tab, so this page documents
//! `WindowStyle` and renders chrome previews for each variant — the
//! standalone `window_layouts` example continues to expose the live
//! switcher.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::presentation::window::WindowStyle;
use tahoe_gpui::foundations::materials::glass_surface;
use tahoe_gpui::foundations::theme::{TahoeTheme, GlassSize, TextStyle, TextStyledExt};

use crate::ComponentGallery;

#[allow(deprecated)]
const STYLES: &[(WindowStyle, &str, &str)] = &[
    (
        WindowStyle::Document,
        "Document",
        "Full 28pt title bar, close + minimize + zoom traffic lights.",
    ),
    (
        WindowStyle::Auxiliary,
        "Auxiliary",
        "Single-task surface — Close button only.",
    ),
    (
        WindowStyle::Settings,
        "Settings",
        "Centered, fixed-size preferences window.",
    ),
    (
        WindowStyle::About,
        "About",
        "Compact non-resizable about window.",
    ),
    (
        WindowStyle::Welcome,
        "Welcome",
        "Centered onboarding window with larger footprint.",
    ),
];

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let chrome = |style: WindowStyle, name: &'static str, body: &'static str| {
        let dot = |color: gpui::Hsla| {
            div()
                .w(px(11.0))
                .h(px(11.0))
                .rounded(px(11.0))
                .bg(color)
        };
        let traffic_lights = match style {
            WindowStyle::Document | WindowStyle::Welcome => div()
                .flex()
                .gap(px(7.0))
                .child(dot(gpui::hsla(0.0, 0.6, 0.55, 1.0)))
                .child(dot(gpui::hsla(0.13, 0.6, 0.55, 1.0)))
                .child(dot(gpui::hsla(0.32, 0.6, 0.55, 1.0))),
            _ => div()
                .flex()
                .gap(px(7.0))
                .child(dot(gpui::hsla(0.0, 0.6, 0.55, 1.0))),
        };

        glass_surface(
            div()
                .w(px(280.0))
                .overflow_hidden()
                .rounded(theme.radius_lg),
            theme,
            GlassSize::Medium,
        )
        .child(
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .h(px(28.0))
                        .px(theme.spacing_sm)
                        .flex()
                        .items_center()
                        .border_b_1()
                        .border_color(theme.border)
                        .child(traffic_lights),
                )
                .child(
                    div()
                        .p(theme.spacing_md)
                        .flex()
                        .flex_col()
                        .gap(theme.spacing_xs)
                        .child(
                            div()
                                .text_style_emphasized(TextStyle::Headline, theme)
                                .text_color(theme.text)
                                .child(name),
                        )
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .child(body),
                        )
                        .child(
                            div()
                                .pt(theme.spacing_xs)
                                .text_style(TextStyle::Caption2, theme)
                                .text_color(theme.text_muted)
                                .child(format!(
                                    "min: {}\u{00d7}{} pt",
                                    style.min_width() as u32,
                                    style.min_height() as u32
                                )),
                        ),
                ),
        )
    };

    div()
        .id("windows-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Windows"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "macOS 26 distinguishes primary windows (Document, Settings, \
                     About, Welcome) from auxiliary windows. The window_layouts \
                     example opens each variant for live inspection; the chrome \
                     previews below summarise the title-bar shape per variant.",
                ),
        )
        .child(
            div()
                .pt(theme.spacing_md)
                .flex()
                .flex_wrap()
                .gap(theme.spacing_md)
                .children(
                    STYLES
                        .iter()
                        .map(|(style, name, body)| chrome(*style, name, body)),
                ),
        )
        .into_any_element()
}
