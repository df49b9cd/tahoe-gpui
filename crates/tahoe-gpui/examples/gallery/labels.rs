//! Labels demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::components::content::label::Label;
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
        .id("labels-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Labels"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A label displays a short amount of text. Labels support \
                     HIG text styles, bold weight, and custom colors.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(Label::new("Large Title").text_style(TextStyle::LargeTitle))
        .child(Label::new("Title 1").text_style(TextStyle::Title1))
        .child(Label::new("Title 2").text_style(TextStyle::Title2))
        .child(Label::new("Title 3").text_style(TextStyle::Title3))
        .child(Label::new("Headline").text_style(TextStyle::Headline))
        .child(Label::new("Body (default)"))
        .child(Label::new("Subheadline").text_style(TextStyle::Subheadline))
        .child(Label::new("Caption 1").text_style(TextStyle::Caption1))
        .child(Label::new("Caption 2").text_style(TextStyle::Caption2))
        .child(div().h(theme.spacing_sm))
        .child(Label::new("Emphasized label").emphasize(true))
        .child(Label::new("Muted label").color(theme.text_muted))
        .child(Label::new("Accent label").color(theme.accent))
        .into_any_element()
}
