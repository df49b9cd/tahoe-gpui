//! Image Views demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Div, Window, div, px};

use tahoe_gpui::components::content::image_view::{ContentMode, ImageView};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let sample_uri = "https://picsum.photos/id/1/400/300";
    let dark_sample_uri = "https://picsum.photos/id/2/400/300";

    div()
        .id("image-views-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Image Views"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A read-only image display with content-mode, corner-radius, \
                     dark-variant, and background options.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        // Content modes
        .child(
            div()
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Content Modes"),
        )
        .child(
            div()
                .flex()
                .gap(theme.spacing_lg)
                .items_start()
                .child(content_mode_card(
                    "Aspect Fit",
                    ImageView::new(sample_uri)
                        .size(px(120.0))
                        .content_mode(ContentMode::AspectFit)
                        .accessibility_label("Aspect fit sample photo"),
                    theme,
                ))
                .child(content_mode_card(
                    "Aspect Fill",
                    ImageView::new(sample_uri)
                        .size(px(120.0))
                        .content_mode(ContentMode::AspectFill)
                        .rounded(px(8.0))
                        .accessibility_label("Aspect fill sample photo"),
                    theme,
                ))
                .child(content_mode_card(
                    "Fill",
                    ImageView::new(sample_uri)
                        .size(px(120.0))
                        .content_mode(ContentMode::Fill)
                        .accessibility_label("Fill sample photo"),
                    theme,
                )),
        )
        .child(div().h(theme.spacing_sm))
        // Corner radius
        .child(
            div()
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Corner Radius"),
        )
        .child(
            div()
                .flex()
                .gap(theme.spacing_lg)
                .items_start()
                .child(content_mode_card(
                    "Rounded",
                    ImageView::new(sample_uri)
                        .size(px(120.0))
                        .rounded(px(12.0))
                        .accessibility_label("Rounded sample photo"),
                    theme,
                ))
                .child(content_mode_card(
                    "Circular",
                    ImageView::new(sample_uri)
                        .size(px(120.0))
                        .circular()
                        .accessibility_label("Circular sample photo"),
                    theme,
                )),
        )
        .child(div().h(theme.spacing_sm))
        // Dark variant
        .child(
            div()
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Dark Variant"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child("Switch appearance to see the dark-variant image."),
        )
        .child(
            div()
                .flex()
                .gap(theme.spacing_lg)
                .items_start()
                .child(content_mode_card(
                    "With dark_uri",
                    ImageView::new(sample_uri)
                        .size(px(120.0))
                        .rounded(px(8.0))
                        .dark_uri(dark_sample_uri)
                        .accessibility_label("Dark-variant sample photo"),
                    theme,
                )),
        )
        .child(div().h(theme.spacing_sm))
        // Background
        .child(
            div()
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Background"),
        )
        .child(
            div()
                .flex()
                .gap(theme.spacing_lg)
                .items_start()
                .child(content_mode_card(
                    "Transparent (default)",
                    ImageView::new(sample_uri)
                        .size(px(120.0))
                        .content_mode(ContentMode::AspectFit)
                        .accessibility_label("Transparent background sample"),
                    theme,
                ))
                .child(content_mode_card(
                    "Opaque",
                    ImageView::new(sample_uri)
                        .size(px(120.0))
                        .content_mode(ContentMode::AspectFit)
                        .opaque()
                        .accessibility_label("Opaque background sample"),
                    theme,
                )),
        )
        .into_any_element()
}

fn content_mode_card(title: &'static str, image: ImageView, theme: &TahoeTheme) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(theme.spacing_xs)
        .child(image)
        .child(
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .child(title),
        )
}
