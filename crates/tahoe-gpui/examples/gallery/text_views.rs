//! Text Views demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::components::content::text_view::TextView;
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
        .id("text-views-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Text Views"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A text view displays read-only, styled text blocks. \
                     Unlike a label, it is designed for multi-line paragraphs.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Body style (default)"),
        )
        .child(TextView::new(
            "The quick brown fox jumps over the lazy dog. \
             This text view uses the default Body text style.",
        ))
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Title 1 style"),
        )
        .child(TextView::new("Large styled heading text").text_style(TextStyle::Title1))
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Caption style"),
        )
        .child(
            TextView::new("Small caption text suitable for footnotes and metadata.")
                .text_style(TextStyle::Caption1),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("With max_lines(2)"),
        )
        .child(
            TextView::new(
                "This text view has max_lines set to 2. While GPUI does not yet \
                 support line clamping natively, the max_lines field is stored for \
                 future use. In practice, wrap in a fixed-height container.",
            )
            .max_lines(2),
        )
        .into_any_element()
}
