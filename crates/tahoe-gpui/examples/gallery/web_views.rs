//! Web Views demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::content::web_view::WebView;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};
use tahoe_gpui::patterns::loading::LoadingState;

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    div()
        .id("web-views-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Web Views"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A web view loads and displays rich web content directly \
                     within your app. This placeholder renders until GPUI \
                     exposes a native web-view surface.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Idle (default)"),
        )
        .child(WebView::new("https://apple.com").size(px(480.0), px(200.0)))
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Loaded"),
        )
        .child(
            WebView::new("https://apple.com")
                .size(px(480.0), px(200.0))
                .loading_state(LoadingState::Loaded),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Loading (indeterminate)"),
        )
        .child(
            WebView::new("https://apple.com")
                .size(px(480.0), px(200.0))
                .loading(true),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Loading with progress"),
        )
        .child(
            WebView::new("https://apple.com")
                .size(px(480.0), px(200.0))
                .loading_state(LoadingState::LoadingWithProgress(0.65)),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Failed"),
        )
        .child(
            WebView::new("https://apple.com")
                .size(px(480.0), px(200.0))
                .loading_state(LoadingState::Failed),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Navigation disabled"),
        )
        .child(
            WebView::new("https://example.com/ad")
                .size(px(480.0), px(200.0))
                .allow_navigation(false),
        )
        .into_any_element()
}
